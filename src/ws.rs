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
                                let room_sockets = sockets.entry(code.clone()).or_insert_with(HashMap::new);
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
                                    if let Some(existing) = room.players.iter_mut().find(|p| p.id == pid) {
                                        existing.connected = true;
                                        existing.conn_id = conn_id.clone();
                                        if !player_name.trim().is_empty() {
                                            existing.name = sanitize_name(&player_name, &existing.name);
                                        }
                                        if let Some(ref a) = avatar {
                                            existing.avatar = sanitize_avatar(Some(a.clone()), &existing.avatar);
                                        }
                                        if let Some(ref c) = color {
                                            existing.color = sanitize_color(Some(c.clone()), &existing.color);
                                        }
                                        is_reconnect = true;
                                    }
                                }

                                if !is_reconnect {
                                    if room.players.len() >= 8 {
                                        let _ = tx.send(Message::Text(
                                            json!({
                                                "type": "error",
                                                "message": "Room is full (max 8 players)"
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
                                        "#3b82f6", "#10b981", "#f59e0b", "#8b5cf6",
                                        "#ec4899", "#06b6d4", "#f97316", "#14b8a6",
                                    ];
                                    let default_avatars = [
                                        "💀", "🪅", "🌵", "🤼", "🎸", "🌶️", "🐆", "☀️",
                                    ];

                                    let p_idx = room.players.len();
                                    let fallback_color = default_colors[p_idx % default_colors.len()];
                                    let fallback_avatar = default_avatars[p_idx % default_avatars.len()];

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
                                    let room_sockets = sockets.entry(code.clone()).or_insert_with(HashMap::new);
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
                                    if let Some(player) = room.players.iter_mut().find(|p| &p.id == pid) {
                                        if let Some(a) = avatar {
                                            player.avatar = sanitize_avatar(Some(a), &player.avatar);
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
                                            room.settings = settings;
                                        } else if room.state == RoomState::Playing {
                                            room.settings.turn_timer_seconds = settings.turn_timer_seconds;
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
                                        let next_timer: u64 = match room.settings.turn_timer_seconds {
                                            0 => 15,
                                            1..=15 => 25,
                                            16..=25 => 40,
                                            _ => 0,
                                        };
                                        room.settings.turn_timer_seconds = next_timer;
                                        room.turn_started_at_secs = crate::game::current_epoch_secs();
                                        if next_timer > 0 {
                                            room.add_log(
                                                "timer_on",
                                                &format!("⏱️ Temporizador cambiado a {}s por turno.", next_timer),
                                                &format!("⏱️ Turn timer changed to {}s per turn.", next_timer),
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
                                                json!({ "type": "error", "message": err }).to_string(),
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
                                                json!({ "type": "error", "message": err }).to_string(),
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
                                "🎉", "🐍", "🪜", "😱", "🎲", "🌮", "🎺", "🌶️", "💀",
                                "¡Aguas con la víbora! 🐍",
                                "¡Sube, compadre! 🪜",
                                "¡Ya merito llego! 🏁",
                                "¡Tírale con fe! 🎲",
                                "¡Salado me quedé! 🧂",
                            ];
                            if !allowed_reactions.contains(&emoji.as_str()) {
                                continue;
                            }

                            let rooms = state.rooms.read().await;
                            if let Some(room) = rooms.get(&room_code) {
                                if let Some(ref pid) = current_player_id {
                                    if let Some(player) = room.players.iter().find(|p| &p.id == pid) {
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

                                    if let Some(pos) = room.players.iter().position(|p| &p.id == pid) {
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
                                        if let Some(next_host) = room.players.iter_mut().find(|p| p.connected) {
                                            next_host.is_host = true;
                                            let h_name = next_host.name.clone();
                                            let h_id = next_host.id.clone();
                                            room.add_log(
                                                "host_change",
                                                &format!("👑 {} es ahora el nuevo anfitrión.", h_name),
                                                &format!("👑 {} is now the new host.", h_name),
                                                Some(h_id),
                                            );
                                        }
                                    }

                                    if room.state == RoomState::Playing && !room.players.is_empty() {
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

/// Background routine to reap abandoned rooms and handle turn timeouts for AFK players
pub async fn start_background_maintenance(state: AppState) {
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(3));
    loop {
        ticker.tick().await;

        let now = current_epoch_secs();
        let mut rooms_to_reap = Vec::new();
        let mut turn_timeouts = Vec::new();

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

                // 2. Check for turn timeout in active games
                if let Some(step) = room.check_turn_timeout() {
                    turn_timeouts.push((code.clone(), step));
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

        // Broadcast turn timeouts if any occurred
        for (code, step) in turn_timeouts {
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
