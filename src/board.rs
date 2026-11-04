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
            assert!(ladder.to > ladder.from, "Ladder must ascend: {} -> {}", ladder.from, ladder.to);
            assert!(ladder.from >= 1 && ladder.to <= 100);
        }

        // Verify all snakes descend
        for snake in &board.snakes {
            assert!(!snake.is_ladder);
            assert!(snake.to < snake.from, "Snake must descend: {} -> {}", snake.from, snake.to);
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
}
