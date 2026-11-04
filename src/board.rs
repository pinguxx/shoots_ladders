use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoardPosition {
    pub tile: u8,
    pub row: u8, // 0..9 (0 is bottom, 9 is top)
    pub col: u8, // 0..9 (0 is left, 9 is right)
    pub x_pct: f32, // 0.0 .. 100.0%
    pub y_pct: f32, // 0.0 .. 100.0% (0% is top)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portal {
    pub from: u8,
    pub to: u8,
    pub is_ladder: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardDefinition {
    pub total_tiles: u8,
    pub ladders: Vec<Portal>,
    pub snakes: Vec<Portal>,
}

impl BoardDefinition {
    pub fn classic() -> Self {
        let ladders = vec![
            Portal { from: 4, to: 14, is_ladder: true },
            Portal { from: 9, to: 31, is_ladder: true },
            Portal { from: 20, to: 38, is_ladder: true },
            Portal { from: 28, to: 84, is_ladder: true },
            Portal { from: 40, to: 59, is_ladder: true },
            Portal { from: 51, to: 67, is_ladder: true },
            Portal { from: 63, to: 81, is_ladder: true },
            Portal { from: 71, to: 91, is_ladder: true },
        ];

        let snakes = vec![
            Portal { from: 17, to: 7, is_ladder: false },
            Portal { from: 54, to: 34, is_ladder: false },
            Portal { from: 62, to: 19, is_ladder: false },
            Portal { from: 64, to: 60, is_ladder: false },
            Portal { from: 87, to: 24, is_ladder: false },
            Portal { from: 93, to: 73, is_ladder: false },
            Portal { from: 95, to: 75, is_ladder: false },
            Portal { from: 99, to: 78, is_ladder: false },
        ];

        Self {
            total_tiles: 100,
            ladders,
            snakes,
        }
    }

    /// Convert tile number 1..=100 into board coordinates.
    /// Tile 0 represents the starting dock ("Salida" / Start).
    pub fn tile_coords(tile: u8) -> BoardPosition {
        if tile == 0 {
            return BoardPosition {
                tile: 0,
                row: 0,
                col: 0,
                x_pct: -6.0,
                y_pct: 95.0,
            };
        }

        let clamped = tile.clamp(1, 100);
        let row = (clamped - 1) / 10;
        let col = if row.is_multiple_of(2) {
            (clamped - 1) % 10
        } else {
            9 - ((clamped - 1) % 10)
        };

        // Screen percentage (0% top-left, 100% bottom-right)
        // Row 0 is at bottom (y ~ 90%..100%), Row 9 is at top (y ~ 0%..10%)
        let x_pct = (col as f32 + 0.5) * 10.0;
        let y_pct = (9.0 - row as f32 + 0.5) * 10.0;

        BoardPosition {
            tile: clamped,
            row,
            col,
            x_pct,
            y_pct,
        }
    }

    /// Check if tile has a portal (ladder or snake)
    pub fn get_portal(&self, tile: u8) -> Option<Portal> {
        if let Some(l) = self.ladders.iter().find(|l| l.from == tile) {
            return Some(l.clone());
        }
        if let Some(s) = self.snakes.iter().find(|s| s.from == tile) {
            return Some(s.clone());
        }
        None
    }
}
