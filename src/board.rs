use crate::piece::{Chip, Player};

pub struct Board {
    pub p1_pieces: u64,
    pub p2_pieces: u64,
    pub dama_pieces: u64,
    pub chips: [Option<Chip>; 64],
    pub current_turn: Player,
}

impl Board {
    pub fn new() -> Self {
        let p1_pieces: u64 = 0x0000000000aa55aa;
        let p2_pieces: u64 = 0x55aa550000000000;

        let mut board = Self {
            p1_pieces,
            p2_pieces,
            dama_pieces: 0,
            chips: [None; 64],
            current_turn: Player::Player1,
        };

        board.init_integer_chips();
        board
    }

    pub fn display(&self) {
        println!();
        println!("     0    1    2    3    4    5    6    7");
        println!("   ┌────┬────┬────┬────┬────┬────┬────┬────┐");

        for row in (0..8).rev() {
            print!(" {} │", row);
            for col in 0..8 {
                let idx = row * 8 + col;
                match self.chips[idx] {
                    Some(ref chip) => {
                        let symbol = match chip.player {
                            Player::Player1 => 'A',
                            Player::Player2 => 'B',
                        };
                        print!("{}{:>3}│", symbol, chip.value);
                    }
                    None => print!("    │"),
                }
            }
            println!(" {}", row);

            if row > 0 {
                println!("   ├────┼────┼────┼────┼────┼────┼────┼────┤");
            }
        }

        println!("   └────┴────┴────┴────┴────┴────┴────┴────┘");
        println!("     0    1    2    3    4    5    6    7");
        println!();
        println!("Turn: {:?}", self.current_turn);
    }

    pub fn make_move(
        &mut self,
        from_row: i32,
        from_col: i32,
        to_row: i32,
        to_col: i32,
    ) -> Result<(), &'static str> {
        if !(0..8).contains(&from_row)
            || !(0..8).contains(&from_col)
            || !(0..8).contains(&to_row)
            || !(0..8).contains(&to_col)
        {
            return Err("Make Move Error: Coordinates are out of bounds.");
        }

        let from_idx: usize = (from_row * 8 + from_col) as usize;
        let to_idx: usize = (to_row * 8 + to_col) as usize;
        let chip = match self.chips[from_idx] {
            Some(c) => {
                if c.player != self.current_turn {
                    return Err("Make Move Error: Opponents pieces cannot be moved.");
                }
                c
            }
            None => return Err("Make Move Error: No piece found at selected start square."),
        };

        if self.chips[to_idx].is_some() {
            return Err("Make Move Error: Destination square is already occupied.");
        }

        let row_diff = to_row - from_row;
        let col_diff = to_col - from_col;

        match (row_diff.abs(), col_diff.abs()) {
            (1, 1) => match self.current_turn {
                Player::Player1 if row_diff != 1 => {
                    return Err(
                        "Make Move Error: Player 1 non-dama pieces can only move forward (increasing row).",
                    );
                }
                Player::Player2 if row_diff != -1 => {
                    return Err(
                        "Make Move Error: Player 2 non-dama pieces can only move forward (decreasing row).",
                    );
                }
                _ => {
                    self.move_chip_data(from_idx, to_idx);
                    self.switch_turn();
                    Ok(())
                }
            },
            (2, 2) => {
                let mid_row = (from_row + to_row) / 2;
                let mid_col = (from_col + to_col) / 2;
                let mid_idx = (mid_row * 8 + mid_col) as usize;

                let jumped_chip = match self.chips[mid_idx] {
                    Some(c) => {
                        if c.player == self.current_turn {
                            return Err("Jump Capture Error: Own pieces cannot be captured");
                        }
                        c
                    }
                    None => return Err("Jump Capture Error: No piece to capture"),
                };

                let captured_value = jumped_chip.value;
                println!("Captured opponent chip with value: {}", captured_value);

                self.chips[mid_idx] = None;
                self.clear_bit(mid_idx, jumped_chip.player);
                self.move_chip_data(from_idx, to_idx);
                self.switch_turn();
                return Ok(());
            }
            _ => Err("Make Move Error: Invalid non-diagonal move"),
        }
    }

    fn move_chip_data(&mut self, from_idx: usize, to_idx: usize) {
        let chip = self.chips[from_idx].take();
        self.chips[to_idx] = chip;

        // Update bitboards
        if self.current_turn == Player::Player1 {
            self.p1_pieces &= !(1u64 << from_idx);
            self.p1_pieces |= 1u64 << to_idx;
        } else {
            self.p2_pieces &= !(1u64 << from_idx);
            self.p2_pieces |= 1u64 << to_idx;
        }
    }

    fn switch_turn(&mut self) {
        self.current_turn = match self.current_turn {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        };
    }

    fn clear_bit(&mut self, idx: usize, player: Player) {
        if player == Player::Player1 {
            self.p1_pieces &= !(1u64 << idx);
        } else {
            self.p2_pieces &= !(1u64 << idx);
        }
    }

    fn init_integer_chips(&mut self) {
        let p1_layout = [
            (0, 1, -11),
            (0, 3, 8),
            (0, 5, -5),
            (0, 7, 2),
            (1, 0, 0),
            (1, 2, -3),
            (1, 4, 10),
            (1, 6, -7),
            (2, 1, -9),
            (2, 3, 6),
            (2, 5, -1),
            (2, 7, 4),
        ];

        for &(row, col, val) in &p1_layout {
            let idx = row * 8 + col;
            self.chips[idx] = Some(Chip::new(Player::Player1, val));
        }

        let p2_layout = [
            (5, 0, 4),
            (5, 2, -1),
            (5, 4, 6),
            (5, 6, -9),
            (6, 1, -7),
            (6, 3, 10),
            (6, 5, -3),
            (6, 7, 0),
            (7, 0, 2),
            (7, 2, -5),
            (7, 4, 8),
            (7, 6, -11),
        ];

        for &(row, col, val) in &p2_layout {
            let idx = row * 8 + col;
            self.chips[idx] = Some(Chip::new(Player::Player2, val));
        }
    }
}
