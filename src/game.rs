use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::board::{BoardDefinition, Portal};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    #[serde(default = "default_dice_count")]
    pub dice_count: u8, // 1 or 2
    #[serde(default = "default_bonus_on_six")]
    pub bonus_on_six: bool, // Roll 6 grants extra turn
    #[serde(default = "default_exact_win")]
    pub exact_win: bool, // Must land exactly on 100 (overshoot bounces)
    #[serde(default = "default_three_sixes")]
    pub three_sixes_penalty: bool, // 3 consecutive sixes returns to square 1
    #[serde(default = "default_turn_timer")]
    pub turn_timer_seconds: u64, // 0 for infinite, or 15, 25, 40
    #[serde(default = "default_board_theme")]
    pub board_theme: String, // "fiesta", "muertos", "selva", "clasico"
}

fn default_dice_count() -> u8 {
    1
}
fn default_bonus_on_six() -> bool {
    true
}
fn default_exact_win() -> bool {
    true
}
fn default_three_sixes() -> bool {
    false
}
fn default_turn_timer() -> u64 {
    0 // 0 = Sin límite / Manual (no autoplay)
}
fn default_board_theme() -> String {
    "fiesta".to_string()
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            dice_count: default_dice_count(),
            bonus_on_six: default_bonus_on_six(),
            exact_win: default_exact_win(),
            three_sixes_penalty: default_three_sixes(),
            turn_timer_seconds: default_turn_timer(),
            board_theme: default_board_theme(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    pub rolls_count: u32,
    pub ladders_climbed: u32,
    pub snakes_hit: u32,
    pub consecutive_sixes: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub color: String,
    pub avatar: String,
    pub position: u8, // 0 = start dock, 1..=100
    pub connected: bool,
    pub is_host: bool,
    #[serde(default)]
    pub conn_id: String,
    #[serde(default)]
    pub stats: PlayerStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RoomState {
    Lobby,
    Playing,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiceRoll {
    pub dice: Vec<u8>,
    pub total: u8,
    pub is_bonus: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveStep {
    pub player_id: String,
    pub player_name: String,
    pub from_tile: u8,
    pub path: Vec<u8>,
    pub landed_tile: u8,
    pub portal_applied: Option<Portal>,
    pub final_tile: u8,
    pub won: bool,
    pub extra_turn: bool,
    pub penalty_applied: bool,
    pub roll: DiceRoll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinnerInfo {
    pub player_id: String,
    pub player_name: String,
    pub avatar: String,
    pub color: String,
    pub total_rolls: u32,
    pub ladders_climbed: u32,
    pub snakes_hit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLog {
    pub id: u64,
    pub kind: String, // "roll", "ladder", "snake", "bounce", "extra_turn", "penalty", "win", "join", "leave"
    pub message_es: String,
    pub message_en: String,
    pub player_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub code: String,
    pub state: RoomState,
    pub board: BoardDefinition,
    pub settings: GameSettings,
    pub players: Vec<Player>,
    pub turn_index: usize,
    pub turn_started_at_secs: u64,
    pub last_activity_secs: u64,
    pub last_roll: Option<DiceRoll>,
    pub last_move: Option<MoveStep>,
    pub winner: Option<WinnerInfo>,
    pub logs: Vec<GameLog>,
    pub round: u32,
    pub log_counter: u64,
}

pub fn generate_room_code() -> String {
    let prefixes = ["SERP", "ESCA", "VIBO", "DADO", "META", "CHUT", "LADD", "MAYA", "AZTE", "FIESTA"];
    let mut rng = rand::thread_rng();
    let prefix = prefixes.choose(&mut rng).unwrap_or(&"SERP");
    let num: u16 = rng.gen_range(10..99);
    format!("{}{}", prefix, num)
}

pub fn current_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl Room {
    pub fn new(code: String, settings: GameSettings) -> Self {
        let now = current_epoch_secs();
        Self {
            code,
            state: RoomState::Lobby,
            board: BoardDefinition::classic(),
            settings,
            players: Vec::new(),
            turn_index: 0,
            turn_started_at_secs: now,
            last_activity_secs: now,
            last_roll: None,
            last_move: None,
            winner: None,
            logs: Vec::new(),
            round: 1,
            log_counter: 0,
        }
    }

    pub fn touch_activity(&mut self) {
        self.last_activity_secs = current_epoch_secs();
    }

    pub fn add_log(&mut self, kind: &str, es: &str, en: &str, player_id: Option<String>) {
        self.log_counter += 1;
        self.logs.push(GameLog {
            id: self.log_counter,
            kind: kind.to_string(),
            message_es: es.to_string(),
            message_en: en.to_string(),
            player_id,
        });

        // Keep last 40 logs to avoid unbounded memory growth
        if self.logs.len() > 40 {
            self.logs.drain(0..self.logs.len() - 40);
        }
    }

    pub fn current_player(&self) -> Option<&Player> {
        if self.players.is_empty() {
            None
        } else {
            self.players.get(self.turn_index % self.players.len())
        }
    }

    pub fn advance_turn(&mut self) {
        if self.players.is_empty() {
            return;
        }

        self.turn_started_at_secs = current_epoch_secs();
        let total = self.players.len();
        for offset in 1..=total {
            let next_idx = (self.turn_index + offset) % total;
            if self.players[next_idx].connected {
                self.turn_index = next_idx;
                return;
            }
        }
        // If no connected players, just increment
        self.turn_index = (self.turn_index + 1) % total;
    }

    pub fn start_game(&mut self) -> Result<(), String> {
        let active_players = self.players.iter().filter(|p| p.connected).count();
        if active_players < 1 {
            return Err("At least 1 player required to start".to_string());
        }

        self.state = RoomState::Playing;
        self.turn_index = 0;
        self.turn_started_at_secs = current_epoch_secs();
        self.touch_activity();
        self.winner = None;
        self.last_roll = None;
        self.last_move = None;
        self.round = 1;

        // Reset player positions and stats
        for p in &mut self.players {
            p.position = 0;
            p.stats = PlayerStats::default();
        }

        self.add_log(
            "start",
            "🎉 ¡El juego ha comenzado! Que la suerte los acompañe.",
            "🎉 The game has started! May luck be on your side.",
            None,
        );

        Ok(())
    }

    pub fn restart_game(&mut self) {
        self.state = RoomState::Lobby;
        self.turn_index = 0;
        self.turn_started_at_secs = current_epoch_secs();
        self.touch_activity();
        self.winner = None;
        self.last_roll = None;
        self.last_move = None;
        self.round += 1;

        for p in &mut self.players {
            p.position = 0;
            p.stats = PlayerStats::default();
        }

        self.add_log(
            "restart",
            "🔄 Nueva partida lista en la sala de espera.",
            "🔄 New game ready in the lobby.",
            None,
        );
    }

    /// Check if current active turn has timed out (for AFK auto-play)
    pub fn check_turn_timeout(&mut self) -> Option<MoveStep> {
        if self.state != RoomState::Playing || self.settings.turn_timer_seconds == 0 {
            return None;
        }

        let now = current_epoch_secs();
        // Allow a 1-second grace window past the timer
        if now > self.turn_started_at_secs + self.settings.turn_timer_seconds {
            if let Some(player) = self.current_player() {
                let pid = player.id.clone();
                return self.roll_and_move(&pid).ok();
            }
        }
        None
    }

    /// Roll dice and process movement for the player whose turn it is.
    pub fn roll_and_move(&mut self, player_id: &str) -> Result<MoveStep, String> {
        if self.state != RoomState::Playing {
            return Err("The game is not currently active".to_string());
        }

        let cur_id = self.current_player().map(|p| p.id.clone());
        if cur_id.as_deref() != Some(player_id) {
            return Err("It is not your turn to roll".to_string());
        }

        self.touch_activity();
        let mut rng = rand::thread_rng();

        // 1. Roll dice
        let dice_count = self.settings.dice_count.clamp(1, 2);
        let mut dice = Vec::new();
        for _ in 0..dice_count {
            dice.push(rng.gen_range(1..=6));
        }
        let total_roll: u8 = dice.iter().sum();

        // Check bonus turn: 6 in 1-die mode, or doubles in 2-dice mode
        let is_bonus = if dice_count == 1 {
            dice[0] == 6
        } else {
            dice[0] == dice[1]
        };

        let roll_obj = DiceRoll {
            dice: dice.clone(),
            total: total_roll,
            is_bonus: is_bonus && self.settings.bonus_on_six,
        };
        self.last_roll = Some(roll_obj.clone());

        // Extract player info
        let (from_tile, player_name, mut consecutive_sixes) = {
            let p = self.current_player().unwrap();
            (p.position, p.name.clone(), p.stats.consecutive_sixes)
        };

        let three_sixes_rule = self.settings.three_sixes_penalty;
        let exact_win_rule = self.settings.exact_win;
        let bonus_on_six_rule = self.settings.bonus_on_six;

        // 2. Check 3-sixes penalty rule
        if three_sixes_rule && is_bonus {
            consecutive_sixes += 1;
            if consecutive_sixes >= 3 {
                // Apply penalty
                let p_idx = self.turn_index % self.players.len();
                self.players[p_idx].stats.rolls_count += 1;
                self.players[p_idx].stats.consecutive_sixes = 0;
                self.players[p_idx].position = 1;

                self.add_log(
                    "penalty",
                    &format!("⚠️ ¡Tres tiros dobles/seises seguidos! {} regresa a la casilla 1.", player_name),
                    &format!("⚠️ Three consecutive sixes! {} goes back to tile 1.", player_name),
                    Some(player_id.to_string()),
                );

                let step = MoveStep {
                    player_id: player_id.to_string(),
                    player_name,
                    from_tile,
                    path: vec![1],
                    landed_tile: 1,
                    portal_applied: None,
                    final_tile: 1,
                    won: false,
                    extra_turn: false,
                    penalty_applied: true,
                    roll: roll_obj,
                };
                self.last_move = Some(step.clone());
                self.advance_turn();
                return Ok(step);
            }
        } else if !is_bonus {
            consecutive_sixes = 0;
        }

        // 3. Compute step-by-step path
        let mut path = Vec::new();
        let landed_tile: u8 = if from_tile + total_roll <= 100 {
            for t in (from_tile + 1)..=(from_tile + total_roll) {
                path.push(t);
            }
            from_tile + total_roll
        } else if exact_win_rule {
            // Bounce back if overshoot
            let overshoot = (from_tile + total_roll) - 100;
            for t in (from_tile + 1)..=100 {
                path.push(t);
            }
            for bounce_step in 1..=overshoot {
                path.push(100 - bounce_step);
            }
            100 - overshoot
        } else {
            // Any roll >= 100 wins immediately
            for t in (from_tile + 1)..=100 {
                path.push(t);
            }
            100
        };

        // 4. Check portals (Escaleras & Serpientes)
        let portal = if landed_tile < 100 {
            self.board.get_portal(landed_tile)
        } else {
            None
        };

        let final_tile = if let Some(ref p) = portal {
            p.to
        } else {
            landed_tile
        };

        let won = final_tile == 100;

        // Update player stats
        let p_idx = self.turn_index % self.players.len();
        self.players[p_idx].stats.rolls_count += 1;
        self.players[p_idx].stats.consecutive_sixes = consecutive_sixes;
        self.players[p_idx].position = final_tile;
        if let Some(ref p) = portal {
            if p.is_ladder {
                self.players[p_idx].stats.ladders_climbed += 1;
            } else {
                self.players[p_idx].stats.snakes_hit += 1;
            }
        }

        // Formulate logs
        let dice_str = if dice.len() == 1 {
            format!("{}", dice[0])
        } else {
            format!("{}+{}={}", dice[0], dice[1], total_roll)
        };

        self.add_log(
            "roll",
            &format!("🎲 {} tiró [{}] y avanzó a la casilla {}.", player_name, dice_str, landed_tile),
            &format!("🎲 {} rolled [{}] and moved to tile {}.", player_name, dice_str, landed_tile),
            Some(player_id.to_string()),
        );

        if let Some(ref p) = portal {
            if p.is_ladder {
                self.add_log(
                    "ladder",
                    &format!("🪜 ¡QUÉ SUERTE! {} subió por una escalera de la casilla {} a la {}.", player_name, p.from, p.to),
                    &format!("🪜 LUCKY! {} climbed a ladder from tile {} to {}.", player_name, p.from, p.to),
                    Some(player_id.to_string()),
                );
            } else {
                self.add_log(
                    "snake",
                    &format!("🐍 ¡AUCH! {} fue mordido por una serpiente y cayó de la casilla {} a la {}.", player_name, p.from, p.to),
                    &format!("🐍 OUCH! {} was bitten by a snake and fell from tile {} to {}.", player_name, p.from, p.to),
                    Some(player_id.to_string()),
                );
            }
        }

        let extra_turn = bonus_on_six_rule && is_bonus && !won;
        if extra_turn {
            self.turn_started_at_secs = current_epoch_secs(); // Reset turn timer for bonus roll
            self.add_log(
                "extra_turn",
                &format!("✨ ¡Tiro extra! {} tiró un seis y vuelve a tirar.", player_name),
                &format!("✨ Extra roll! {} rolled a six and gets another turn.", player_name),
                Some(player_id.to_string()),
            );
        }

        if won {
            self.state = RoomState::Finished;
            let p_ref = &self.players[p_idx];
            let winner_info = WinnerInfo {
                player_id: player_id.to_string(),
                player_name: player_name.clone(),
                avatar: p_ref.avatar.clone(),
                color: p_ref.color.clone(),
                total_rolls: p_ref.stats.rolls_count,
                ladders_climbed: p_ref.stats.ladders_climbed,
                snakes_hit: p_ref.stats.snakes_hit,
            };
            self.winner = Some(winner_info);

            self.add_log(
                "win",
                &format!("🏆 ¡VICTORIA! ¡{} llegó a la meta (100) y es el Gran Campeón!", player_name),
                &format!("🏆 VICTORY! {} reached tile 100 and won the game!", player_name),
                Some(player_id.to_string()),
            );
        } else if !extra_turn {
            self.advance_turn();
        }

        let step = MoveStep {
            player_id: player_id.to_string(),
            player_name,
            from_tile,
            path,
            landed_tile,
            portal_applied: portal,
            final_tile,
            won,
            extra_turn,
            penalty_applied: false,
            roll: roll_obj,
        };

        self.last_move = Some(step.clone());
        Ok(step)
    }
}
