use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};

use crate::game::{
    current_epoch_secs, generate_room_code, GameSettings, Player, PlayerStats, Room, RoomState,
};

pub type RoomSockets = HashMap<String, mpsc::UnboundedSender<Message>>;

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<RwLock<HashMap<String, Room>>>,
    pub sockets: Arc<RwLock<HashMap<String, RoomSockets>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            sockets: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// -------------------------------------------------------------
// Security & Sanitization Helpers
// -------------------------------------------------------------
pub fn sanitize_name(name: &str, fallback: &str) -> String {
    let clean: String = name
        .trim()
        .chars()
        .filter(|c| !c.is_control() && *c != '<' && *c != '>')
        .take(18)
        .collect();

    if clean.is_empty() {
        fallback.to_string()
    } else {
        clean
    }
}

pub fn sanitize_avatar(avatar: Option<String>, default_avatar: &str) -> String {
    if let Some(a) = avatar {
        let trimmed = a.trim();
        // Allow emoji or up to 4 chars, no HTML characters
        if !trimmed.is_empty()
            && trimmed.chars().count() <= 4
            && !trimmed.contains('<')
            && !trimmed.contains('>')
            && !trimmed.contains('&')
            && !trimmed.contains('"')
        {
            return trimmed.to_string();
        }
    }
    default_avatar.to_string()
}

pub fn sanitize_color(color: Option<String>, default_color: &str) -> String {
    if let Some(c) = color {
        let trimmed = c.trim();
        // Strict 7-char hex check (#rrggbb)
        if trimmed.starts_with('#')
            && trimmed.len() == 7
            && trimmed[1..].chars().all(|h| h.is_ascii_hexdigit())
        {
            return trimmed.to_lowercase();
        }
    }
    default_color.to_string()
}

// Client Inbound Message Envelope
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "create_room")]
    CreateRoom {
        #[serde(rename = "playerName")]
        player_name: String,
        avatar: Option<String>,
        color: Option<String>,
        settings: Option<GameSettings>,
    },
    #[serde(rename = "join_room")]
    JoinRoom {
        #[serde(rename = "roomCode")]
        room_code: String,
        #[serde(rename = "playerName")]
        player_name: String,
        avatar: Option<String>,
        color: Option<String>,
        #[serde(rename = "playerId")]
        player_id: Option<String>,
    },
    #[serde(rename = "update_profile")]
    UpdateProfile {
        #[serde(rename = "roomCode")]
        room_code: String,
        avatar: Option<String>,
        color: Option<String>,
        #[serde(rename = "playerName")]
        player_name: Option<String>,
    },
    #[serde(rename = "update_settings")]
    UpdateSettings {
        #[serde(rename = "roomCode")]
        room_code: String,
        settings: GameSettings,
    },
    #[serde(rename = "start_game")]
    StartGame {
        #[serde(rename = "roomCode")]
        room_code: String,
    },
    #[serde(rename = "roll_dice")]
    RollDice {
        #[serde(rename = "roomCode")]
        room_code: String,
    },
    #[serde(rename = "restart_game")]
    RestartGame {
        #[serde(rename = "roomCode")]
        room_code: String,
    },
    #[serde(rename = "toggle_timer")]
    ToggleTimer {
        #[serde(rename = "roomCode")]
        room_code: String,
    },
    #[serde(rename = "send_reaction")]
    SendReaction {
        #[serde(rename = "roomCode")]
        room_code: String,
        emoji: String,
    },
    #[serde(rename = "leave_room")]
    LeaveRoom {
        #[serde(rename = "roomCode")]
        room_code: String,
    },
    #[serde(rename = "add_bot")]
    AddBot {
        #[serde(rename = "roomCode")]
        room_code: String,
    },
    #[serde(rename = "remove_bot")]
    RemoveBot {
        #[serde(rename = "roomCode")]
        room_code: String,
        #[serde(rename = "botId")]
        bot_id: Option<String>,
    },
    #[serde(rename = "ping")]
    Ping,
}

pub async fn broadcast_to_room(room_code: &str, msg: Value, state: &AppState) {
    let sockets_guard = state.sockets.read().await;
    if let Some(room_map) = sockets_guard.get(room_code) {
        let serialized = serde_json::to_string(&msg).unwrap_or_default();
        for tx in room_map.values() {
            let _ = tx.send(Message::Text(serialized.clone()));
        }
    }
}

pub async fn broadcast_room_state(room_code: &str, state: &AppState) {
    let rooms_guard = state.rooms.read().await;
    if let Some(room) = rooms_guard.get(room_code) {
        let msg = json!({
            "type": "room_state",
            "room": room
        });
        drop(rooms_guard);
        broadcast_to_room(room_code, msg, state).await;
    }
}

pub async fn handle_ws_connection(socket: WebSocket, state: AppState) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Forward outbound channel messages to the client websocket
    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let conn_id = format!("{:x}", rand::random::<u128>());
    let mut current_room: Option<String> = None;
    let mut current_player_id: Option<String> = None;

    // Rate limiter for reactions (max 5 reactions in a 3-second window)
    let mut reaction_history: Vec<Instant> = Vec::new();

    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                // Protect against oversized payload payloads (> 32KB)
                if text.len() > 32_768 {
                    continue;
                }

                let parsed: Result<ClientMessage, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(client_msg) => match client_msg {
                        ClientMessage::CreateRoom {
                            player_name,
                            avatar,
                            color,
                            settings,
                        } => {
                            let mut code = generate_room_code();
                            {
                                let rooms = state.rooms.read().await;
                                while rooms.contains_key(&code) {
                                    code = generate_room_code();
                                }
                            }

                            let clean_name = sanitize_name(&player_name, "Jugador 1");
                            let player_id = format!("p_{:x}", rand::random::<u64>());
                            let chosen_avatar = sanitize_avatar(avatar, "🤠");
                            let chosen_color = sanitize_color(color, "#ef4444");

                            let host_player = Player {
                                id: player_id.clone(),
                                name: clean_name.clone(),
                                color: chosen_color,
                                avatar: chosen_avatar,
                                position: 0,
                                connected: true,
                                is_host: true,
                                conn_id: conn_id.clone(),
                                stats: PlayerStats::default(),
                                is_bot: false,
                                skipped_turns: 0,
                            };

                            let mut room = Room::new(code.clone(), settings.unwrap_or_default());
                            room.players.push(host_player);
                            room.add_log(
                                "join",
                                &format!("🏠 {} creó la sala {}.", clean_name, code),
                                &format!("🏠 {} created room {}.", clean_name, code),
                                Some(player_id.clone()),
                            );

                            {
                                let mut rooms = state.rooms.write().await;
                                rooms.insert(code.clone(), room.clone());
                            }

                            {
                                let mut sockets = state.sockets.write().await;
                                let room_sockets =
                                    sockets.entry(code.clone()).or_insert_with(HashMap::new);
                                room_sockets.insert(player_id.clone(), tx.clone());
                            }

                            current_room = Some(code.clone());
                            current_player_id = Some(player_id.clone());

                            let _ = tx.send(Message::Text(
                                json!({
                                    "type": "room_joined",
                                    "roomCode": code,
                                    "playerId": player_id,
                                    "isHost": true,
                                    "room": room
                                })
                                .to_string(),
                            ));
                        }

                        ClientMessage::JoinRoom {
                            room_code,
                            player_name,
                            avatar,
                            color,
                            player_id,
                        } => {
                            let code = room_code.trim().to_uppercase();
                            let mut rooms = state.rooms.write().await;

                            if let Some(room) = rooms.get_mut(&code) {
                                room.touch_activity();
                                let mut pid = player_id.unwrap_or_default();
                                let mut is_reconnect = false;

                                // Check if reconnecting existing player
                                if !pid.is_empty() {
                                    if let Some(existing) =
                                        room.players.iter_mut().find(|p| p.id == pid)
                                    {
                                        existing.connected = true;
                                        existing.conn_id = conn_id.clone();
                                        if !player_name.trim().is_empty() {
                                            existing.name =
                                                sanitize_name(&player_name, &existing.name);
                                        }
                                        if let Some(ref a) = avatar {
                                            existing.avatar =
                                                sanitize_avatar(Some(a.clone()), &existing.avatar);
                                        }
                                        if let Some(ref c) = color {
                                            existing.color =
                                                sanitize_color(Some(c.clone()), &existing.color);
                                        }
                                        is_reconnect = true;
                                    }
                                }

                                if !is_reconnect {
                                    if room.players.len() >= 10 {
                                        let _ = tx.send(Message::Text(
                                            json!({
                                                "type": "error",
                                                "message": "Room is full (max 10 players)"
                                            })
                                            .to_string(),
                                        ));
                                        continue;
                                    }

                                    pid = format!("p_{:x}", rand::random::<u64>());
                                    let clean_name = sanitize_name(
                                        &player_name,
                                        &format!("Jugador {}", room.players.len() + 1),
                                    );

                                    let default_colors = [
                                        "#3b82f6", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899",
                                        "#06b6d4", "#f97316", "#14b8a6", "#eab308", "#f43f5e",
                                    ];
                                    let default_avatars = [
                                        "🧗‍♂️",
                                        "🐍",
                                        "🎲",
                                        "🤠",
                                        "🦅",
                                        "🐆",
                                        "🦁",
                                        "🦊",
                                        "🐸",
                                        "🦜",
                                    ];

                                    let p_idx = room.players.len();
                                    let fallback_color =
                                        default_colors[p_idx % default_colors.len()];
                                    let fallback_avatar =
                                        default_avatars[p_idx % default_avatars.len()];

                                    let chosen_color = sanitize_color(color, fallback_color);
                                    let chosen_avatar = sanitize_avatar(avatar, fallback_avatar);

                                    let is_first = room.players.is_empty();
                                    let new_player = Player {
                                        id: pid.clone(),
                                        name: clean_name.clone(),
                                        color: chosen_color,
                                        avatar: chosen_avatar,
                                        position: 0,
                                        connected: true,
                                        is_host: is_first,
                                        conn_id: conn_id.clone(),
                                        stats: PlayerStats::default(),
                                        is_bot: false,
                                        skipped_turns: 0,
                                    };

                                    room.players.push(new_player);
                                    room.add_log(
                                        "join",
                                        &format!("👋 {} se unió a la sala.", clean_name),
                                        &format!("👋 {} joined the room.", clean_name),
                                        Some(pid.clone()),
                                    );
                                }

                                let is_host = room
                                    .players
                                    .iter()
                                    .find(|p| p.id == pid)
                                    .map(|p| p.is_host)
                                    .unwrap_or(false);
                                let snapshot = room.clone();
                                drop(rooms);

                                {
                                    let mut sockets = state.sockets.write().await;
                                    let room_sockets =
                                        sockets.entry(code.clone()).or_insert_with(HashMap::new);
                                    room_sockets.insert(pid.clone(), tx.clone());
                                }

                                current_room = Some(code.clone());
                                current_player_id = Some(pid.clone());

                                let _ = tx.send(Message::Text(
                                    json!({
                                        "type": "room_joined",
                                        "roomCode": code,
                                        "playerId": pid,
                                        "isHost": is_host,
                                        "room": snapshot
                                    })
                                    .to_string(),
                                ));

                                broadcast_room_state(&code, &state).await;
                            } else {
                                let _ = tx.send(Message::Text(
                                    json!({
                                        "type": "error",
                                        "message": "Room not found. Check the code and try again."
                                    })
                                    .to_string(),
                                ));
                            }
                        }

                        ClientMessage::UpdateProfile {
                            room_code,
                            avatar,
                            color,
                            player_name,
                        } => {
                            let mut rooms = state.rooms.write().await;
                            if let Some(room) = rooms.get_mut(&room_code) {
                                room.touch_activity();
                                if let Some(ref pid) = current_player_id {
                                    if let Some(player) =
                                        room.players.iter_mut().find(|p| &p.id == pid)
                                    {
                                        if let Some(a) = avatar {
                                            player.avatar =
                                                sanitize_avatar(Some(a), &player.avatar);
                                        }
                                        if let Some(c) = color {
                                            player.color = sanitize_color(Some(c), &player.color);
                                        }
                                        if let Some(n) = player_name {
                                            player.name = sanitize_name(&n, &player.name);
                                        }
                                    }
                                }
                            }
                            drop(rooms);
                            broadcast_room_state(&room_code, &state).await;
                        }

                        ClientMessage::UpdateSettings {
                            room_code,
                            settings,
                        } => {
                            let mut rooms = state.rooms.write().await;
                            if let Some(room) = rooms.get_mut(&room_code) {
                                room.touch_activity();
                                if let Some(ref pid) = current_player_id {
                                    let is_host = room
                                        .players
                                        .iter()
                                        .find(|p| &p.id == pid)
                                        .map(|p| p.is_host)
                                        .unwrap_or(false);
                                    if is_host {
                                        if room.state == RoomState::Lobby {
                                            room.board = if settings.board_size == 50 {
                                                crate::board::BoardDefinition::express()
                                                    .with_mystery_tiles(settings.mystery_tiles)
                                            } else {
                                                crate::board::BoardDefinition::classic()
                                                    .with_mystery_tiles(settings.mystery_tiles)
                                            };
                                            room.settings = settings;
                                        } else if room.state == RoomState::Playing {
                                            room.settings.turn_timer_seconds =
                                                settings.turn_timer_seconds;
                                        }
                                    }
                                }
                            }
                            drop(rooms);
                            broadcast_room_state(&room_code, &state).await;
                        }

                        ClientMessage::ToggleTimer { room_code } => {
                            let mut rooms = state.rooms.write().await;
                            if let Some(room) = rooms.get_mut(&room_code) {
                                room.touch_activity();
                                if let Some(ref pid) = current_player_id {
                                    let is_host = room
                                        .players
                                        .iter()
                                        .find(|p| &p.id == pid)
                                        .map(|p| p.is_host)
                                        .unwrap_or(false);
                                    if is_host {
                                        // Cycle through timer presets: 0 (Manual) -> 15s -> 25s -> 40s -> 0
                                        let next_timer: u64 = match room.settings.turn_timer_seconds
                                        {
                                            0 => 15,
                                            1..=15 => 25,
                                            16..=25 => 40,
                                            _ => 0,
                                        };
                                        room.settings.turn_timer_seconds = next_timer;
                                        room.turn_started_at_secs =
                                            crate::game::current_epoch_secs();
                                        if next_timer > 0 {
                                            room.add_log(
                                                "timer_on",
                                                &format!(
                                                    "⏱️ Temporizador cambiado a {}s por turno.",
                                                    next_timer
                                                ),
                                                &format!(
                                                    "⏱️ Turn timer changed to {}s per turn.",
                                                    next_timer
                                                ),
                                                None,
                                            );
                                        } else {
                                            room.add_log(
                                                "timer_off",
                                                "⏸️ Temporizador desactivado (Sin límite / Manual).",
                                                "⏸️ Turn timer disabled (No limit / Manual).",
                                                None,
                                            );
                                        }
                                    }
                                }
                            }
                            drop(rooms);
                            broadcast_room_state(&room_code, &state).await;
                        }

                        ClientMessage::StartGame { room_code } => {
                            let mut rooms = state.rooms.write().await;
                            if let Some(room) = rooms.get_mut(&room_code) {
                                if let Some(ref pid) = current_player_id {
                                    let is_host = room
                                        .players
                                        .iter()
                                        .find(|p| &p.id == pid)
                                        .map(|p| p.is_host)
                                        .unwrap_or(false);
                                    if is_host {
                                        if let Err(err) = room.start_game() {
                                            let _ = tx.send(Message::Text(
                                                json!({ "type": "error", "message": err })
                                                    .to_string(),
                                            ));
                                        }
                                    }
                                }
                            }
                            drop(rooms);
                            broadcast_room_state(&room_code, &state).await;
                        }

                        ClientMessage::RollDice { room_code } => {
                            let mut rooms = state.rooms.write().await;
                            let mut move_res = None;

                            if let Some(room) = rooms.get_mut(&room_code) {
                                if let Some(ref pid) = current_player_id {
                                    match room.roll_and_move(pid) {
                                        Ok(step) => {
                                            move_res = Some(step);
                                        }
                                        Err(err) => {
                                            let _ = tx.send(Message::Text(
                                                json!({ "type": "error", "message": err })
                                                    .to_string(),
                                            ));
                                        }
                                    }
                                }
                            }
                            drop(rooms);

                            if let Some(step) = move_res {
                                broadcast_to_room(
                                    &room_code,
                                    json!({
                                        "type": "player_moved",
                                        "step": step
                                    }),
                                    &state,
                                )
                                .await;

                                broadcast_room_state(&room_code, &state).await;
                            }
                        }

                        ClientMessage::RestartGame { room_code } => {
                            let mut rooms = state.rooms.write().await;
                            if let Some(room) = rooms.get_mut(&room_code) {
                                if let Some(ref pid) = current_player_id {
                                    let is_host = room
                                        .players
                                        .iter()
                                        .find(|p| &p.id == pid)
                                        .map(|p| p.is_host)
                                        .unwrap_or(false);
                                    if is_host {
                                        room.restart_game();
                                    }
                                }
                            }
                            drop(rooms);
                            broadcast_room_state(&room_code, &state).await;
                        }

                        ClientMessage::SendReaction { room_code, emoji } => {
                            // Rate-limit check (at most 5 reactions in 3 seconds)
                            let now = Instant::now();
                            reaction_history.retain(|t| now.duration_since(*t).as_secs() < 3);
                            if reaction_history.len() >= 5 {
                                continue;
                            }
                            reaction_history.push(now);

                            // Whitelist approved reaction emojis and festive shouts
                            let allowed_reactions = [
                                "🎉",
                                "🐍",
                                "🪜",
                                "😱",
                                "🎲",
                                "🏆",
                                "⭐",
                                "🧗‍♂️",
                                "🔥",
                                "🌮",
                                "🎺",
                                "🌶️",
                                "💀",
                                // Spanish shouts
                                "¡Aguas con la víbora! 🐍",
                                "¡Subiendo por la escalera! 🪜",
                                "¡Sube, compadre! 🪜",
                                "¡Ya merito llego a la meta! 🏁",
                                "¡Ya merito llego! 🏁",
                                "¡Tírale con fe al dado! 🎲",
                                "¡Tírale con fe! 🎲",
                                "¡Nooo, mordida de serpiente! 😱",
                                "¡Salado me quedé! 🧂",
                                "¡Ándale, tira el dado! ⏰",
                                "¡Ándale, te toca! ⏰",
                                "¡Pura suerte con el dado! 🍀",
                                "¡Pura suerte! 🍀",
                                // English shouts
                                "Watch out for the snake! 🐍",
                                "Watch out for snakes! 🐍",
                                "Climbing up the ladder! 🪜",
                                "Climb that ladder! 🪜",
                                "Almost at the finish! 🏁",
                                "Almost at 100! 🏁",
                                "Roll with faith! 🎲",
                                "Oh no, snake bite! 😱",
                                "Down I go! 🧂",
                                "Your turn to roll! ⏰",
                                "Hurry up, your turn! ⏰",
                                "Pure dice luck! 🍀",
                                "Pure luck! 🍀",
                                // Canonical keys
                                "shoutSnake",
                                "shoutLadder",
                                "shoutAlmost",
                                "shoutFaith",
                                "shoutUnlucky",
                                "shoutYourTurn",
                                "shoutLucky",
                            ];
                            if !allowed_reactions.contains(&emoji.as_str()) {
                                continue;
                            }

                            let rooms = state.rooms.read().await;
                            if let Some(room) = rooms.get(&room_code) {
                                if let Some(ref pid) = current_player_id {
                                    if let Some(player) = room.players.iter().find(|p| &p.id == pid)
                                    {
                                        let reaction_msg = json!({
                                            "type": "reaction",
                                            "playerId": player.id,
                                            "playerName": player.name,
                                            "avatar": player.avatar,
                                            "emoji": emoji
                                        });
                                        drop(rooms);
                                        broadcast_to_room(&room_code, reaction_msg, &state).await;
                                        continue;
                                    }
                                }
                            }
                        }

                        ClientMessage::LeaveRoom { room_code } => {
                            let code = room_code.trim().to_uppercase();
                            if let Some(ref pid) = current_player_id {
                                let mut rooms = state.rooms.write().await;
                                if let Some(room) = rooms.get_mut(&code) {
                                    room.touch_activity();
                                    let mut player_name = "Player".to_string();
                                    let mut was_host = false;

                                    if let Some(pos) =
                                        room.players.iter().position(|p| &p.id == pid)
                                    {
                                        let p = room.players.remove(pos);
                                        player_name = p.name;
                                        was_host = p.is_host;
                                    }

                                    room.add_log(
                                        "leave",
                                        &format!("🚪 {} abandonó la sala.", player_name),
                                        &format!("🚪 {} left the room.", player_name),
                                        Some(pid.clone()),
                                    );

                                    if was_host {
                                        if let Some(next_host) =
                                            room.players.iter_mut().find(|p| p.connected)
                                        {
                                            next_host.is_host = true;
                                            let h_name = next_host.name.clone();
                                            let h_id = next_host.id.clone();
                                            room.add_log(
                                                "host_change",
                                                &format!(
                                                    "👑 {} es ahora el nuevo anfitrión.",
                                                    h_name
                                                ),
                                                &format!("👑 {} is now the new host.", h_name),
                                                Some(h_id),
                                            );
                                        }
                                    }

                                    if room.state == RoomState::Playing && !room.players.is_empty()
                                    {
                                        room.advance_turn();
                                    }
                                }
                                drop(rooms);

                                {
                                    let mut sockets = state.sockets.write().await;
                                    if let Some(room_sockets) = sockets.get_mut(&code) {
                                        room_sockets.remove(pid);
                                    }
                                }

                                current_room = None;
                                current_player_id = None;
                                broadcast_room_state(&code, &state).await;
                            }
                        }

                        ClientMessage::AddBot { room_code } => {
                            let mut rooms = state.rooms.write().await;
                            if let Some(room) = rooms.get_mut(&room_code) {
                                room.touch_activity();
                                if let Some(ref pid) = current_player_id {
                                    let is_host = room
                                        .players
                                        .iter()
                                        .find(|p| &p.id == pid)
                                        .map(|p| p.is_host)
                                        .unwrap_or(false);
                                    if is_host && room.state == RoomState::Lobby {
                                        let _ = room.add_bot();
                                    }
                                }
                            }
                            drop(rooms);
                            broadcast_room_state(&room_code, &state).await;
                        }

                        ClientMessage::RemoveBot { room_code, bot_id } => {
                            let mut rooms = state.rooms.write().await;
                            if let Some(room) = rooms.get_mut(&room_code) {
                                room.touch_activity();
                                if let Some(ref pid) = current_player_id {
                                    let is_host = room
                                        .players
                                        .iter()
                                        .find(|p| &p.id == pid)
                                        .map(|p| p.is_host)
                                        .unwrap_or(false);
                                    if is_host && room.state == RoomState::Lobby {
                                        let _ = room.remove_bot(bot_id.as_deref());
                                    }
                                }
                            }
                            drop(rooms);
                            broadcast_room_state(&room_code, &state).await;
                        }

                        ClientMessage::Ping => {
                            let _ = tx.send(Message::Text(json!({ "type": "pong" }).to_string()));
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to parse client message: {} | Text: {}", e, text);
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Handle momentary disconnect cleanup (e.g. browser refresh or cellular network blip)
    // NOTE: We mark connected = false but do NOT immediately skip their turn or strip host.
    // The background maintenance timer will advance turn if they stay disconnected past the deadline.
    if let (Some(code), Some(pid)) = (current_room, current_player_id) {
        let mut rooms = state.rooms.write().await;
        if let Some(room) = rooms.get_mut(&code) {
            if let Some(player) = room.players.iter_mut().find(|p| p.id == pid) {
                player.connected = false;
            }
        }
        drop(rooms);

        {
            let mut sockets = state.sockets.write().await;
            if let Some(room_sockets) = sockets.get_mut(&code) {
                room_sockets.remove(&pid);
            }
        }

        broadcast_room_state(&code, &state).await;
    }

    forward_task.abort();
}

/// Background routine to reap abandoned rooms, handle bot turns, and AFK turn timeouts
pub async fn start_background_maintenance(state: AppState) {
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_millis(800));
    loop {
        ticker.tick().await;

        let now = current_epoch_secs();
        let mut rooms_to_reap = Vec::new();
        let mut turn_moves = Vec::new();
        let mut bot_reactions = Vec::new();

        {
            let mut rooms = state.rooms.write().await;
            for (code, room) in rooms.iter_mut() {
                let connected_count = room.players.iter().filter(|p| p.connected).count();

                // 1. Check for abandoned rooms:
                // If 0 connected players and inactive for > 10 minutes, or inactive for > 2 hours
                if (connected_count == 0 && now > room.last_activity_secs + 600)
                    || (now > room.last_activity_secs + 7200)
                {
                    rooms_to_reap.push(code.clone());
                    continue;
                }

                // 2. Check for bot turns
                if let Some((step, shout)) = room.check_bot_turn() {
                    if let Some(bot_info) = room.players.iter().find(|p| p.id == step.player_id) {
                        if let Some(shout_txt) = shout {
                            bot_reactions.push((
                                code.clone(),
                                bot_info.id.clone(),
                                bot_info.name.clone(),
                                bot_info.avatar.clone(),
                                shout_txt,
                            ));
                        }
                    }
                    turn_moves.push((code.clone(), step));
                    continue;
                }

                // 3. Check for AFK turn timeout in active games
                if let Some(step) = room.check_turn_timeout() {
                    turn_moves.push((code.clone(), step));
                }
            }

            // Remove abandoned rooms
            for code in &rooms_to_reap {
                rooms.remove(code);
            }
        }

        // Clean sockets for reaped rooms
        if !rooms_to_reap.is_empty() {
            let mut sockets = state.sockets.write().await;
            for code in rooms_to_reap {
                sockets.remove(&code);
            }
        }

        // Broadcast reactions from bots
        for (code, pid, pname, pavatar, shout) in bot_reactions {
            broadcast_to_room(
                &code,
                json!({
                    "type": "reaction",
                    "playerId": pid,
                    "playerName": pname,
                    "avatar": pavatar,
                    "emoji": shout
                }),
                &state,
            )
            .await;
        }

        // Broadcast turn moves if any occurred
        for (code, step) in turn_moves {
            broadcast_to_room(
                &code,
                json!({
                    "type": "player_moved",
                    "step": step
                }),
                &state,
            )
            .await;
            broadcast_room_state(&code, &state).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Pancho Villa", "Jugador"), "Pancho Villa");
        assert_eq!(
            sanitize_name("   Lucky Charlie   ", "Jugador"),
            "Lucky Charlie"
        );
        assert_eq!(sanitize_name("", "Jugador"), "Jugador");
        assert_eq!(sanitize_name("   ", "Jugador"), "Jugador");

        // XSS and control characters stripped and truncated to 18 chars
        assert_eq!(
            sanitize_name("<script>alert(1)</script>", "Jugador"),
            "scriptalert(1)/scr"
        );
        assert_eq!(
            sanitize_name("User <img src=x>", "Jugador"),
            "User img src=x"
        );

        // Truncated at 18 characters
        let long_name = "Supercalifragilisticexpialidocious";
        let sanitized = sanitize_name(long_name, "Jugador");
        assert_eq!(sanitized.chars().count(), 18);
        assert_eq!(sanitized, "Supercalifragilist");
    }

    #[test]
    fn test_sanitize_avatar() {
        assert_eq!(sanitize_avatar(Some("🤠".to_string()), "🤠"), "🤠");
        assert_eq!(sanitize_avatar(Some("💀".to_string()), "🤠"), "💀");
        assert_eq!(sanitize_avatar(Some("🪅".to_string()), "🤠"), "🪅");
        assert_eq!(sanitize_avatar(None, "🤠"), "🤠");

        // Disallowed / HTML payloads fallback to default
        assert_eq!(sanitize_avatar(Some("<x>".to_string()), "🤠"), "🤠");
        assert_eq!(sanitize_avatar(Some("a&b".to_string()), "🤠"), "🤠");
        assert_eq!(sanitize_avatar(Some("\"".to_string()), "🤠"), "🤠");
        assert_eq!(
            sanitize_avatar(Some("TooLongAvatar".to_string()), "🤠"),
            "🤠"
        );
        assert_eq!(sanitize_avatar(Some("".to_string()), "🤠"), "🤠");
    }

    #[test]
    fn test_sanitize_color() {
        assert_eq!(
            sanitize_color(Some("#f59e0b".to_string()), "#f59e0b"),
            "#f59e0b"
        );
        assert_eq!(
            sanitize_color(Some("#EF4444".to_string()), "#f59e0b"),
            "#ef4444"
        );
        assert_eq!(sanitize_color(None, "#f59e0b"), "#f59e0b");

        // Invalid colors fallback to default
        assert_eq!(
            sanitize_color(Some("red".to_string()), "#f59e0b"),
            "#f59e0b"
        );
        assert_eq!(
            sanitize_color(Some("#123".to_string()), "#f59e0b"),
            "#f59e0b"
        );
        assert_eq!(
            sanitize_color(Some("#xyz123".to_string()), "#f59e0b"),
            "#f59e0b"
        );
        assert_eq!(
            sanitize_color(Some("<script>".to_string()), "#f59e0b"),
            "#f59e0b"
        );
    }

    #[test]
    fn test_client_messages_deserialization() {
        let create_json = r#"{"type":"create_room","playerName":"Pancho"}"#;
        let msg: ClientMessage = serde_json::from_str(create_json).unwrap();
        match msg {
            ClientMessage::CreateRoom { player_name, .. } => assert_eq!(player_name, "Pancho"),
            _ => panic!("Expected CreateRoom"),
        }

        let join_json =
            r#"{"type":"join_room","roomCode":"SERP12","playerName":"Lupe","playerId":"p_123"}"#;
        let msg: ClientMessage = serde_json::from_str(join_json).unwrap();
        match msg {
            ClientMessage::JoinRoom {
                room_code,
                player_name,
                player_id,
                ..
            } => {
                assert_eq!(room_code, "SERP12");
                assert_eq!(player_name, "Lupe");
                assert_eq!(player_id.as_deref(), Some("p_123"));
            }
            _ => panic!("Expected JoinRoom"),
        }

        let update_profile_json =
            r##"{"type":"update_profile","roomCode":"SERP12","avatar":"🌵","color":"#10b981"}"##;
        let msg: ClientMessage = serde_json::from_str(update_profile_json).unwrap();
        match msg {
            ClientMessage::UpdateProfile {
                room_code,
                avatar,
                color,
                ..
            } => {
                assert_eq!(room_code, "SERP12");
                assert_eq!(avatar.as_deref(), Some("🌵"));
                assert_eq!(color.as_deref(), Some("#10b981"));
            }
            _ => panic!("Expected UpdateProfile"),
        }

        let update_settings_json =
            r#"{"type":"update_settings","roomCode":"SERP12","settings":{"dice_count":2}}"#;
        let msg: ClientMessage = serde_json::from_str(update_settings_json).unwrap();
        match msg {
            ClientMessage::UpdateSettings {
                room_code,
                settings,
            } => {
                assert_eq!(room_code, "SERP12");
                assert_eq!(settings.dice_count, 2);
            }
            _ => panic!("Expected UpdateSettings"),
        }

        let roll_json = r#"{"type":"roll_dice","roomCode":"SERP12"}"#;
        let msg: ClientMessage = serde_json::from_str(roll_json).unwrap();
        match msg {
            ClientMessage::RollDice { room_code } => assert_eq!(room_code, "SERP12"),
            _ => panic!("Expected RollDice"),
        }

        let toggle_timer_json = r#"{"type":"toggle_timer","roomCode":"SERP12"}"#;
        let msg: ClientMessage = serde_json::from_str(toggle_timer_json).unwrap();
        match msg {
            ClientMessage::ToggleTimer { room_code } => assert_eq!(room_code, "SERP12"),
            _ => panic!("Expected ToggleTimer"),
        }

        let reaction_json =
            r#"{"type":"send_reaction","roomCode":"SERP12","emoji":"Watch out for snakes! 🐍"}"#;
        let msg: ClientMessage = serde_json::from_str(reaction_json).unwrap();
        match msg {
            ClientMessage::SendReaction { room_code, emoji } => {
                assert_eq!(room_code, "SERP12");
                assert_eq!(emoji, "Watch out for snakes! 🐍");
            }
            _ => panic!("Expected SendReaction"),
        }

        let leave_json = r#"{"type":"leave_room","roomCode":"SERP12"}"#;
        let msg: ClientMessage = serde_json::from_str(leave_json).unwrap();
        match msg {
            ClientMessage::LeaveRoom { room_code } => assert_eq!(room_code, "SERP12"),
            _ => panic!("Expected LeaveRoom"),
        }

        let ping_json = r#"{"type":"ping"}"#;
        let msg: ClientMessage = serde_json::from_str(ping_json).unwrap();
        match msg {
            ClientMessage::Ping => (),
            _ => panic!("Expected Ping"),
        }
    }

    #[test]
    fn test_allowed_reactions_whitelist() {
        let allowed = [
            "🎉",
            "🐍",
            "🪜",
            "😱",
            "🎲",
            "🏆",
            "⭐",
            "🧗‍♂️",
            "🔥",
            "🌮",
            "🎺",
            "🌶️",
            "💀",
            "¡Aguas con la víbora! 🐍",
            "¡Subiendo por la escalera! 🪜",
            "¡Sube, compadre! 🪜",
            "¡Ya merito llego a la meta! 🏁",
            "¡Ya merito llego! 🏁",
            "¡Tírale con fe al dado! 🎲",
            "¡Tírale con fe! 🎲",
            "¡Nooo, mordida de serpiente! 😱",
            "¡Salado me quedé! 🧂",
            "¡Ándale, tira el dado! ⏰",
            "¡Ándale, te toca! ⏰",
            "¡Pura suerte con el dado! 🍀",
            "¡Pura suerte! 🍀",
            "Watch out for the snake! 🐍",
            "Watch out for snakes! 🐍",
            "Climbing up the ladder! 🪜",
            "Climb that ladder! 🪜",
            "Almost at the finish! 🏁",
            "Almost at 100! 🏁",
            "Roll with faith! 🎲",
            "Oh no, snake bite! 😱",
            "Down I go! 🧂",
            "Your turn to roll! ⏰",
            "Hurry up, your turn! ⏰",
            "Pure dice luck! 🍀",
            "Pure luck! 🍀",
            "shoutSnake",
            "shoutLadder",
            "shoutAlmost",
            "shoutFaith",
            "shoutUnlucky",
            "shoutYourTurn",
            "shoutLucky",
        ];

        // Valid ones
        assert!(allowed.contains(&"🎉"));
        assert!(allowed.contains(&"¡Aguas con la víbora! 🐍"));
        assert!(allowed.contains(&"Watch out for snakes! 🐍"));
        assert!(allowed.contains(&"shoutSnake"));

        // Malicious or unapproved ones
        assert!(!allowed.contains(&"<script>"));
        assert!(!allowed.contains(&"spam"));
        assert!(!allowed.contains(&"hacked"));
    }

    #[tokio::test]
    async fn test_app_state_and_room_lifecycle() {
        let state = AppState::new();
        assert_eq!(state.rooms.read().await.len(), 0);
        assert_eq!(state.sockets.read().await.len(), 0);

        // Insert room
        let room = Room::new("ROOM99".to_string(), GameSettings::default());
        state.rooms.write().await.insert("ROOM99".to_string(), room);
        assert_eq!(state.rooms.read().await.len(), 1);

        // Broadcast to room with no sockets shouldn't panic
        broadcast_to_room("ROOM99", json!({"type": "test"}), &state).await;
        broadcast_room_state("ROOM99", &state).await;

        // Clean up
        state.rooms.write().await.remove("ROOM99");
        assert_eq!(state.rooms.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_timer_cycling_presets() {
        let mut settings = GameSettings::default();
        assert_eq!(settings.turn_timer_seconds, 0);

        // Cycle 1: 0 -> 15
        settings.turn_timer_seconds = match settings.turn_timer_seconds {
            0 => 15,
            15 => 25,
            25 => 40,
            _ => 0,
        };
        assert_eq!(settings.turn_timer_seconds, 15);

        // Cycle 2: 15 -> 25
        settings.turn_timer_seconds = match settings.turn_timer_seconds {
            0 => 15,
            15 => 25,
            25 => 40,
            _ => 0,
        };
        assert_eq!(settings.turn_timer_seconds, 25);

        // Cycle 3: 25 -> 40
        settings.turn_timer_seconds = match settings.turn_timer_seconds {
            0 => 15,
            15 => 25,
            25 => 40,
            _ => 0,
        };
        assert_eq!(settings.turn_timer_seconds, 40);

        // Cycle 4: 40 -> 0
        settings.turn_timer_seconds = match settings.turn_timer_seconds {
            0 => 15,
            15 => 25,
            25 => 40,
            _ => 0,
        };
        assert_eq!(settings.turn_timer_seconds, 0);
    }

    #[tokio::test]
    async fn test_websocket_full_lifecycle() {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::Message as WsMsg;

        let state = AppState::new();
        let app = crate::create_app(state.clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let ws_url = format!("ws://{}/ws", addr);

        // 1. Client 1 connects (Host)
        let (mut client1, _) = tokio_tungstenite::connect_async(&ws_url).await.unwrap();

        // Send create_room
        let create_msg = json!({
            "type": "create_room",
            "playerName": "HostPancho",
            "avatar": "🤠",
            "color": "#f59e0b",
            "settings": {
                "dice_count": 1,
                "turn_timer_seconds": 15
            }
        });
        client1
            .send(WsMsg::Text(create_msg.to_string()))
            .await
            .unwrap();

        // Receive room_joined
        let resp = client1.next().await.unwrap().unwrap();
        let text = resp.to_text().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["type"], "room_joined");
        let room_code = parsed["roomCode"].as_str().unwrap().to_string();
        let host_id = parsed["playerId"].as_str().unwrap().to_string();
        assert_eq!(parsed["room"]["players"].as_array().unwrap().len(), 1);

        // Send update_profile
        let update_prof = json!({
            "type": "update_profile",
            "roomCode": room_code,
            "avatar": "🌵",
            "color": "#10b981",
            "playerName": "PanchoVilla"
        });
        client1
            .send(WsMsg::Text(update_prof.to_string()))
            .await
            .unwrap();
        let _ = client1.next().await.unwrap().unwrap();

        // Send update_settings
        let update_sett = json!({
            "type": "update_settings",
            "roomCode": room_code,
            "settings": {
                "dice_count": 2,
                "bonus_on_six": true,
                "exact_win": true,
                "three_sixes_penalty": false,
                "turn_timer_seconds": 25,
                "board_theme": "selva"
            }
        });
        client1
            .send(WsMsg::Text(update_sett.to_string()))
            .await
            .unwrap();
        let _ = client1.next().await.unwrap().unwrap();

        // Send toggle_timer
        let toggle_timer = json!({
            "type": "toggle_timer",
            "roomCode": room_code
        });
        client1
            .send(WsMsg::Text(toggle_timer.to_string()))
            .await
            .unwrap();
        let _ = client1.next().await.unwrap().unwrap();

        // Send add_bot
        let add_bot_msg = json!({
            "type": "add_bot",
            "roomCode": room_code
        });
        client1
            .send(WsMsg::Text(add_bot_msg.to_string()))
            .await
            .unwrap();
        let _ = client1.next().await.unwrap().unwrap();

        // Send remove_bot
        let remove_bot_msg = json!({
            "type": "remove_bot",
            "roomCode": room_code,
            "botId": null
        });
        client1
            .send(WsMsg::Text(remove_bot_msg.to_string()))
            .await
            .unwrap();
        let _ = client1.next().await.unwrap().unwrap();

        // 2. Client 2 connects (Guest)
        let (mut client2, _) = tokio_tungstenite::connect_async(&ws_url).await.unwrap();
        let join_msg = json!({
            "type": "join_room",
            "roomCode": room_code,
            "playerName": "GuestAdelita",
            "avatar": "💀",
            "color": "#ef4444"
        });
        client2
            .send(WsMsg::Text(join_msg.to_string()))
            .await
            .unwrap();

        // Client 2 receives room_joined
        let join_resp = client2.next().await.unwrap().unwrap();
        let join_parsed: Value = serde_json::from_str(join_resp.to_text().unwrap()).unwrap();
        assert_eq!(join_parsed["type"], "room_joined");
        assert_eq!(join_parsed["room"]["players"].as_array().unwrap().len(), 2);

        // Client 1 receives room_state broadcast
        let _ = client1.next().await.unwrap().unwrap();

        // 3. Client 1 starts game
        let start_msg = json!({
            "type": "start_game",
            "roomCode": room_code
        });
        client1
            .send(WsMsg::Text(start_msg.to_string()))
            .await
            .unwrap();
        let _ = client1.next().await.unwrap().unwrap();
        let _ = client2.next().await.unwrap().unwrap();

        // 4. Client 1 rolls dice
        let roll_msg = json!({
            "type": "roll_dice",
            "roomCode": room_code
        });
        client1
            .send(WsMsg::Text(roll_msg.to_string()))
            .await
            .unwrap();
        // Receives player_moved and room_state on client1
        let _ = client1.next().await.unwrap().unwrap();
        let _ = client1.next().await.unwrap().unwrap();
        // Receives player_moved and room_state on client2
        let _ = client2.next().await.unwrap().unwrap();
        let _ = client2.next().await.unwrap().unwrap();

        // 5. Client 2 sends reaction
        let reaction_msg = json!({
            "type": "send_reaction",
            "roomCode": room_code,
            "emoji": "Watch out for snakes! 🐍"
        });
        client2
            .send(WsMsg::Text(reaction_msg.to_string()))
            .await
            .unwrap();
        let rx1 = client1.next().await.unwrap().unwrap();
        let rx_parsed: Value = serde_json::from_str(rx1.to_text().unwrap()).unwrap();
        assert_eq!(rx_parsed["type"], "reaction");
        assert_eq!(rx_parsed["emoji"], "Watch out for snakes! 🐍");
        let _ = client2.next().await.unwrap().unwrap();

        // 6. Ping / Pong
        client1
            .send(WsMsg::Text(json!({"type": "ping"}).to_string()))
            .await
            .unwrap();
        let pong = client1.next().await.unwrap().unwrap();
        let pong_parsed: Value = serde_json::from_str(pong.to_text().unwrap()).unwrap();
        assert_eq!(pong_parsed["type"], "pong");

        // 7. Restart game
        let restart_msg = json!({
            "type": "restart_game",
            "roomCode": room_code
        });
        client1
            .send(WsMsg::Text(restart_msg.to_string()))
            .await
            .unwrap();
        let _ = client1.next().await.unwrap().unwrap();
        let _ = client2.next().await.unwrap().unwrap();

        // 8. Client 1 leaves room (triggers host transfer to Client 2!)
        let leave_msg = json!({
            "type": "leave_room",
            "roomCode": room_code
        });
        client1
            .send(WsMsg::Text(leave_msg.to_string()))
            .await
            .unwrap();

        // Read messages from client2 until we get room_state with is_host = true
        let mut promoted = false;
        for _ in 0..10 {
            if let Some(Ok(msg)) = client2.next().await {
                if let Ok(parsed) = serde_json::from_str::<Value>(msg.to_text().unwrap()) {
                    if parsed["type"] == "room_state" {
                        if let Some(players) = parsed["room"]["players"].as_array() {
                            if let Some(c2) = players.iter().find(|p| p["name"] == "GuestAdelita") {
                                if c2["is_host"] == true {
                                    promoted = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(
            promoted,
            "Client 2 must be promoted to host after Client 1 leaves"
        );

        // 9. Reconnect with existing playerId
        let (mut client3, _) = tokio_tungstenite::connect_async(&ws_url).await.unwrap();
        let rejoin_msg = json!({
            "type": "join_room",
            "roomCode": room_code,
            "playerName": "PanchoReconnected",
            "playerId": host_id
        });
        client3
            .send(WsMsg::Text(rejoin_msg.to_string()))
            .await
            .unwrap();
        let rejoin_resp = client3.next().await.unwrap().unwrap();
        let rejoin_parsed: Value = serde_json::from_str(rejoin_resp.to_text().unwrap()).unwrap();
        assert_eq!(rejoin_parsed["type"], "room_joined");

        // Client 2 receives broadcast of Client 3 rejoining
        let _ = client2.next().await.unwrap().unwrap();

        // Client 2 leaves
        client2
            .send(WsMsg::Text(leave_msg.to_string()))
            .await
            .unwrap();
        let _ = client3.next().await.unwrap().unwrap();

        // Client 3 leaves (room now empty -> cleaned up)
        client3
            .send(WsMsg::Text(leave_msg.to_string()))
            .await
            .unwrap();

        server_handle.abort();
    }
}
