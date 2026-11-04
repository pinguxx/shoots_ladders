use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoardPosition {
    pub tile: u8,
    pub row: u8,    // 0..9 (0 is bottom, 9 is top)
    pub col: u8,    // 0..9 (0 is left, 9 is right)
    pub x_pct: f32, // 0.0 .. 100.0%
    pub y_pct: f32, // 0.0 .. 100.0% (0% is top)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portal {
    pub from: u8,
    pub to: u8,
    pub is_ladder: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MysteryKind {
    #[serde(alias = "taco")]
    Lightning, // +3 tiles boost (Lightning Dash / Impulso Relámpago ⚡)
    #[serde(alias = "chile")]
    Star, // +5 tiles boost (Super Star / Estrella de la Suerte 🌟)
    #[serde(alias = "pinata")]
    LuckyDie, // Extra roll immediately! (Lucky Die / Dado Dorado 🎲)
    #[serde(alias = "nopal")]
    Trap, // Trap: miss 1 turn (Pitfall Trap / Casilla Trampa 🕳️)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysteryTile {
    pub tile: u8,
    pub kind: MysteryKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardDefinition {
    pub total_tiles: u8,
    pub ladders: Vec<Portal>,
    pub snakes: Vec<Portal>,
    #[serde(default)]
    pub mystery_tiles: Vec<MysteryTile>,
}

impl BoardDefinition {
    pub fn classic() -> Self {
        let ladders = vec![
            Portal {
                from: 4,
                to: 14,
                is_ladder: true,
            },
            Portal {
                from: 9,
                to: 31,
                is_ladder: true,
            },
            Portal {
                from: 20,
                to: 38,
                is_ladder: true,
            },
            Portal {
                from: 28,
                to: 84,
                is_ladder: true,
            },
            Portal {
                from: 40,
                to: 59,
                is_ladder: true,
            },
            Portal {
                from: 51,
                to: 67,
                is_ladder: true,
            },
            Portal {
                from: 63,
                to: 81,
                is_ladder: true,
            },
            Portal {
                from: 71,
                to: 91,
                is_ladder: true,
            },
        ];

        let snakes = vec![
            Portal {
                from: 17,
                to: 7,
                is_ladder: false,
            },
            Portal {
                from: 54,
                to: 34,
                is_ladder: false,
            },
            Portal {
                from: 62,
                to: 19,
                is_ladder: false,
            },
            Portal {
                from: 64,
                to: 60,
                is_ladder: false,
            },
            Portal {
                from: 87,
                to: 24,
                is_ladder: false,
            },
            Portal {
                from: 93,
                to: 73,
                is_ladder: false,
            },
            Portal {
                from: 95,
                to: 75,
                is_ladder: false,
            },
            Portal {
                from: 99,
                to: 78,
                is_ladder: false,
            },
        ];

        Self {
            total_tiles: 100,
            ladders,
            snakes,
            mystery_tiles: Vec::new(),
        }
    }

    pub fn express() -> Self {
        let ladders = vec![
            Portal {
                from: 3,
                to: 12,
                is_ladder: true,
            },
            Portal {
                from: 14,
                to: 28,
                is_ladder: true,
            },
            Portal {
                from: 22,
                to: 37,
                is_ladder: true,
            },
            Portal {
                from: 31,
                to: 46,
                is_ladder: true,
            },
        ];

        let snakes = vec![
            Portal {
                from: 18,
                to: 6,
                is_ladder: false,
            },
            Portal {
                from: 32,
                to: 16,
                is_ladder: false,
            },
            Portal {
                from: 43,
                to: 25,
                is_ladder: false,
            },
            Portal {
                from: 49,
                to: 35,
                is_ladder: false,
            },
        ];

        Self {
            total_tiles: 50,
            ladders,
            snakes,
            mystery_tiles: Vec::new(),
        }
    }

    pub fn with_mystery_tiles(mut self, enabled: bool) -> Self {
        if enabled {
            if self.total_tiles == 50 {
                self.mystery_tiles = vec![
                    MysteryTile {
                        tile: 15,
                        kind: MysteryKind::Lightning,
                    },
                    MysteryTile {
                        tile: 21,
                        kind: MysteryKind::Trap,
                    },
                    MysteryTile {
                        tile: 27,
                        kind: MysteryKind::Star,
                    },
                    MysteryTile {
                        tile: 39,
                        kind: MysteryKind::LuckyDie,
                    },
                ];
            } else {
                self.mystery_tiles = vec![
                    MysteryTile {
                        tile: 33,
                        kind: MysteryKind::Lightning,
                    },
                    MysteryTile {
                        tile: 44,
                        kind: MysteryKind::Trap,
                    },
                    MysteryTile {
                        tile: 55,
                        kind: MysteryKind::Star,
                    },
                    MysteryTile {
                        tile: 77,
                        kind: MysteryKind::LuckyDie,
                    },
                ];
            }
        } else {
            self.mystery_tiles.clear();
        }
        self
    }

    /// Convert tile number into board coordinates.
    pub fn tile_coords_for(tile: u8, total_tiles: u8) -> BoardPosition {
        let max_tile = if total_tiles == 50 { 50 } else { 100 };
        let total_rows = (max_tile / 10) as f32; // 5.0 or 10.0

        if tile == 0 {
            return BoardPosition {
                tile: 0,
                row: 0,
                col: 0,
                x_pct: -6.0,
                y_pct: (total_rows - 0.5) * (100.0 / total_rows),
            };
        }

        let clamped = tile.clamp(1, max_tile);
        let row = (clamped - 1) / 10;
        let col = if row % 2 == 0 {
            (clamped - 1) % 10
        } else {
            9 - ((clamped - 1) % 10)
        };

        let x_pct = (col as f32 + 0.5) * 10.0;
        let y_pct = (total_rows - 1.0 - row as f32 + 0.5) * (100.0 / total_rows);

        BoardPosition {
            tile: clamped,
            row,
            col,
            x_pct,
            y_pct,
        }
    }

    pub fn tile_coords(tile: u8) -> BoardPosition {
        Self::tile_coords_for(tile, 100)
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

    /// Check if tile has a mystery tile powerup
    pub fn get_mystery_tile(&self, tile: u8) -> Option<MysteryTile> {
        self.mystery_tiles.iter().find(|m| m.tile == tile).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_classic_definition() {
        let board = BoardDefinition::classic();
        assert_eq!(board.total_tiles, 100);
        assert_eq!(board.ladders.len(), 8);
        assert_eq!(board.snakes.len(), 8);

        // Verify all ladders ascend
        for ladder in &board.ladders {
            assert!(ladder.is_ladder);
            assert!(
                ladder.to > ladder.from,
                "Ladder must ascend: {} -> {}",
                ladder.from,
                ladder.to
            );
            assert!(ladder.from >= 1 && ladder.to <= 100);
        }

        // Verify all snakes descend
        for snake in &board.snakes {
            assert!(!snake.is_ladder);
            assert!(
                snake.to < snake.from,
                "Snake must descend: {} -> {}",
                snake.from,
                snake.to
            );
            assert!(snake.to >= 1 && snake.from <= 100);
        }
    }

    #[test]
    fn test_tile_coords_start_dock() {
        let dock = BoardDefinition::tile_coords(0);
        assert_eq!(dock.tile, 0);
        assert_eq!(dock.row, 0);
        assert_eq!(dock.col, 0);
        assert_eq!(dock.x_pct, -6.0);
        assert_eq!(dock.y_pct, 95.0);
    }

    #[test]
    fn test_tile_coords_zigzag_boustrophedon() {
        // Row 0 (bottom, left to right): tiles 1..=10
        let t1 = BoardDefinition::tile_coords(1);
        assert_eq!(t1.row, 0);
        assert_eq!(t1.col, 0);
        assert_eq!(t1.x_pct, 5.0);
        assert_eq!(t1.y_pct, 95.0);

        let t10 = BoardDefinition::tile_coords(10);
        assert_eq!(t10.row, 0);
        assert_eq!(t10.col, 9);
        assert_eq!(t10.x_pct, 95.0);
        assert_eq!(t10.y_pct, 95.0);

        // Row 1 (second row, right to left): tiles 11..=20
        let t11 = BoardDefinition::tile_coords(11);
        assert_eq!(t11.row, 1);
        assert_eq!(t11.col, 9);
        assert_eq!(t11.x_pct, 95.0);
        assert_eq!(t11.y_pct, 85.0);

        let t20 = BoardDefinition::tile_coords(20);
        assert_eq!(t20.row, 1);
        assert_eq!(t20.col, 0);
        assert_eq!(t20.x_pct, 5.0);
        assert_eq!(t20.y_pct, 85.0);

        // Row 9 (top row, right to left): tiles 91..=100
        let t91 = BoardDefinition::tile_coords(91);
        assert_eq!(t91.row, 9);
        assert_eq!(t91.col, 9);
        assert_eq!(t91.x_pct, 95.0);
        assert_eq!(t91.y_pct, 5.0);

        let t100 = BoardDefinition::tile_coords(100);
        assert_eq!(t100.row, 9);
        assert_eq!(t100.col, 0);
        assert_eq!(t100.x_pct, 5.0);
        assert_eq!(t100.y_pct, 5.0);

        // Tile clamping above 100
        let clamped = BoardDefinition::tile_coords(150);
        assert_eq!(clamped.tile, 100);
        assert_eq!(clamped.row, 9);
        assert_eq!(clamped.col, 0);
    }

    #[test]
    fn test_get_portal_ladders_and_snakes() {
        let board = BoardDefinition::classic();

        // Check each ladder
        let ladder_tiles = [4, 9, 20, 28, 40, 51, 63, 71];
        let ladder_destinations = [14, 31, 38, 84, 59, 67, 81, 91];
        for (from, to) in ladder_tiles.iter().zip(ladder_destinations.iter()) {
            let portal = board.get_portal(*from).expect("Ladder must exist");
            assert!(portal.is_ladder);
            assert_eq!(portal.from, *from);
            assert_eq!(portal.to, *to);
        }

        // Check each snake
        let snake_tiles = [17, 54, 62, 64, 87, 93, 95, 99];
        let snake_tails = [7, 34, 19, 60, 24, 73, 75, 78];
        for (from, to) in snake_tiles.iter().zip(snake_tails.iter()) {
            let portal = board.get_portal(*from).expect("Snake must exist");
            assert!(!portal.is_ladder);
            assert_eq!(portal.from, *from);
            assert_eq!(portal.to, *to);
        }

        // Tiles without portals
        assert!(board.get_portal(1).is_none());
        assert!(board.get_portal(2).is_none());
        assert!(board.get_portal(50).is_none());
        assert!(board.get_portal(100).is_none());
    }

    #[test]
    fn test_board_express_and_mystery_tiles() {
        let express = BoardDefinition::express().with_mystery_tiles(true);
        assert_eq!(express.total_tiles, 50);
        assert_eq!(express.ladders.len(), 4);
        assert_eq!(express.snakes.len(), 4);
        assert_eq!(express.mystery_tiles.len(), 4);

        // Check mystery tiles
        assert_eq!(
            express.get_mystery_tile(15).unwrap().kind,
            MysteryKind::Lightning
        );
        assert_eq!(
            express.get_mystery_tile(21).unwrap().kind,
            MysteryKind::Trap
        );
        assert_eq!(
            express.get_mystery_tile(27).unwrap().kind,
            MysteryKind::Star
        );
        assert_eq!(
            express.get_mystery_tile(39).unwrap().kind,
            MysteryKind::LuckyDie
        );
        assert!(express.get_mystery_tile(1).is_none());

        // Check coords for 50-tile board (5 rows)
        let c1 = BoardDefinition::tile_coords_for(1, 50);
        assert_eq!(c1.row, 0);
        assert_eq!(c1.col, 0);
        let c50 = BoardDefinition::tile_coords_for(50, 50);
        assert_eq!(c50.row, 4);
        assert_eq!(c50.col, 9); // Row 4 (even) is left-to-right, so 50 is at col 9!
    }
}
