use crate::operator::Operator;
use crate::piece::{Chip, Player};

pub struct Board {
    pub p1_pieces: u64,
    pub p2_pieces: u64,
    pub p1_score: i32,
    pub p2_score: i32,
    pub dama_pieces: u64,
    pub operators: [Operator; 64],
    pub chips: [Option<Chip>; 64],
    pub current_turn: Player,
    pub forced_piece: Option<(usize, usize)>,
}

impl Board {
    pub fn new() -> Self {
        let p1_pieces: u64 = 0x0000000000aa55aa;
        let p2_pieces: u64 = 0x55aa550000000000;

        let mut board = Self {
            p1_pieces,
            p2_pieces,
            p1_score: 0,
            p2_score: 0,
            dama_pieces: 0,
            chips: [None; 64],
            operators: [Operator::Add; 64],
            current_turn: Player::Player1,
            forced_piece: None,
        };

        board.init_operators();
        board.init_integer_chips();
        board
    }

    pub fn display(&self) {
        println!();
        println!("      0     1     2     3     4     5     6     7");
        println!("   ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐");

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
                        let dama_flag = if chip.is_dama { 'D' } else { ' ' };
                        print!("{}{}{:>3}│", symbol, dama_flag, chip.value);
                    }
                    None => print!("     │"),
                }
            }
            println!(" {}", row);

            if row > 0 {
                println!("   ├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤");
            }
        }

        println!("   └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘");
        println!("      0     1     2     3     4     5     6     7");
        println!();
    }

    pub fn player_has_any_capture(&self, player: Player) -> bool {
        let pieces = match player {
            Player::Player1 => self.p1_pieces,
            Player::Player2 => self.p2_pieces,
        };

        let mut bb = pieces;
        while bb != 0 {
            let from_idx = bb.trailing_zeros() as i32;
            let row = from_idx / 8;
            let col = from_idx % 8;

            if self.piece_has_any_capture(row, col) {
                return true;
            }
            bb &= bb - 1;
        }
        false
    }

    pub fn piece_has_any_capture(&self, row: i32, col: i32) -> bool {
        let from_idx = (row * 8 + col) as usize;
        let piece_bit = 1u64 << from_idx;

        if ((self.p1_pieces | self.p2_pieces) & piece_bit) == 0 {
            return false;
        }

        let is_dama = (self.dama_pieces & piece_bit) != 0;
        let is_p1 = (self.p1_pieces & piece_bit) != 0;
        let opponent_pieces = if is_p1 {
            self.p2_pieces
        } else {
            self.p1_pieces
        };
        let occupied = self.p1_pieces | self.p2_pieces;

        let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

        for &(dr, dc) in &directions {
            if is_dama {
                let mut step = 1;
                let mut found_opponent = false;

                loop {
                    let r = row + dr * step;
                    let c = col + dc * step;
                    if !(0..8).contains(&r) || !(0..8).contains(&c) {
                        break;
                    }
                    let idx = (r * 8 + c) as usize;
                    let bit = 1u64 << idx;

                    if (occupied & bit) != 0 {
                        if (opponent_pieces & bit) != 0 && !found_opponent {
                            found_opponent = true;
                        } else {
                            break;
                        }
                    } else if found_opponent {
                        return true;
                    }
                    step += 1;
                }
            } else {
                let target_row = row + dr * 2;
                let target_col = col + dc * 2;
                if (0..8).contains(&target_row) && (0..8).contains(&target_col) {
                    let mid_row = row + dr;
                    let mid_col = col + dc;
                    let mid_idx = (mid_row * 8 + mid_col) as usize;
                    let target_idx = (target_row * 8 + target_col) as usize;

                    let mid_bit = 1u64 << mid_idx;
                    let target_bit = 1u64 << target_idx;

                    if (occupied & target_bit) == 0 && (opponent_pieces & mid_bit) != 0 {
                        return true;
                    }
                }
            }
        }
        false
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

        if let Some((f_row, f_col)) = self.forced_piece {
            if f_row != from_row as usize || f_col != from_col as usize {
                return Err(
                    "Capture Chain Error: You must continue chaining with the active capturing piece.",
                );
            }
        } else {
            let global_capture_exists = self.player_has_any_capture(self.current_turn);
            let row_diff = (to_row - from_row).abs();
            let col_diff = (to_col - from_col).abs();
            let is_attempting_capture = row_diff >= 2 && col_diff >= 2 && row_diff == col_diff;

            if global_capture_exists && !is_attempting_capture {
                return Err(
                    "Rule Error: A capture is available! You are forced to make a capture move.",
                );
            }
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

        if row_diff.abs() != col_diff.abs() {
            return Err("Make Move Error: Moves must be diagonal.");
        }

        let step_r = row_diff.signum();
        let step_c = col_diff.signum();
        let dist = row_diff.abs();
        let is_dama = (self.dama_pieces & (1u64 << from_idx)) != 0;

        // 1. SIMPLE SLIDE MOVE
        if dist == 1 || (dist > 1 && is_dama && !self.piece_has_any_capture(from_row, from_col)) {
            if dist == 1 {
                if self.forced_piece.is_some() {
                    return Err(
                        "Chain Error: You cannot make a simple step while in an active capture chain.",
                    );
                }
                match self.current_turn {
                    Player::Player1 if !is_dama && row_diff != 1 => {
                        return Err(
                            "Make Move Error: Player 1 non-dama pieces can only move forward.",
                        );
                    }
                    Player::Player2 if !is_dama && row_diff != -1 => {
                        return Err(
                            "Make Move Error: Player 2 non-dama pieces can only move forward.",
                        );
                    }
                    _ => {}
                }
                self.move_chip_data(from_idx, to_idx);
                self.check_promotion(to_row, to_idx);
                self.switch_turn();
                return Ok(());
            } else {
                // Long range Dama slide
                if !self.is_path_clear(from_row, from_col, to_row, to_col, None) {
                    return Err("Make Move Error: Dama path is blocked.");
                }
                self.move_chip_data(from_idx, to_idx);
                self.switch_turn();
                return Ok(());
            }
        }

        // 2. CAPTURE MOVE
        let mut found_jumped = None;
        let mut curr_r = from_row + step_r;
        let mut curr_c = from_col + step_c;

        while curr_r != to_row {
            let idx = (curr_r * 8 + curr_c) as usize;
            let bit = 1u64 << idx;
            if ((self.p1_pieces | self.p2_pieces) & bit) != 0 {
                let enemy_pieces = match self.current_turn {
                    Player::Player1 => self.p2_pieces,
                    Player::Player2 => self.p1_pieces,
                };
                if (enemy_pieces & bit) == 0 {
                    return Err("Jump Capture Error: Cannot jump over own pieces.");
                }
                if found_jumped.is_some() {
                    return Err(
                        "Jump Capture Error: Cannot jump over multiple pieces in one jump.",
                    );
                }
                found_jumped = Some((idx, bit));
            }
            curr_r += step_r;
            curr_c += step_c;
        }

        let (jumped_idx, jumped_bit) = match found_jumped {
            Some(j) => j,
            None => return Err("Jump Capture Error: No opponent piece found to jump over."),
        };

        if !is_dama && dist != 2 {
            return Err("Make Move Error: Non-dama pieces can only jump exactly 2 squares.");
        }

        // Ensure landing path from the jumped piece to target destination is clear
        let jumped_r = (jumped_idx / 8) as i32;
        let jumped_c = (jumped_idx % 8) as i32;
        if !self.is_path_clear(jumped_r, jumped_c, to_row, to_col, Some(from_idx)) {
            return Err("Jump Capture Error: Landing path after capture is blocked.");
        }

        let jumped_chip = self.chips[jumped_idx].unwrap();
        let operator = self.operators[to_idx];
        let mut points = operator.apply(chip.value, jumped_chip.value);

        // Apply Dama multiplier logic if applicable
        let target_is_dama = (self.dama_pieces & jumped_bit) != 0;
        if is_dama && target_is_dama {
            points *= 4;
        } else if is_dama || target_is_dama {
            points *= 2;
        }

        match self.current_turn {
            Player::Player1 => self.p1_score += points,
            Player::Player2 => self.p2_score += points,
        }

        self.chips[jumped_idx] = None;
        self.clear_bit(jumped_idx, jumped_chip.player);
        self.move_chip_data(from_idx, to_idx);
        self.check_promotion(to_row, to_idx);

        // --- CHAIN HANDLING ---
        if self.piece_has_any_capture(to_row, to_col) {
            self.forced_piece = Some((to_row as usize, to_col as usize));
            Ok(())
        } else {
            self.forced_piece = None;
            self.switch_turn();
            Ok(())
        }
    }

    fn is_path_clear(&self, r1: i32, c1: i32, r2: i32, c2: i32, ignore_idx: Option<usize>) -> bool {
        let dr = (r2 - r1).signum();
        let dc = (c2 - c1).signum();
        let mut curr_r = r1 + dr;
        let mut curr_c = c1 + dc;

        let occupied = self.p1_pieces | self.p2_pieces;
        while curr_r != r2 {
            let idx = (curr_r * 8 + curr_c) as usize;
            let bit = 1u64 << idx;
            if Some(idx) != ignore_idx && (occupied & bit) != 0 {
                return false;
            }
            curr_r += dr;
            curr_c += dc;
        }
        true
    }

    fn check_promotion(&mut self, row: i32, idx: usize) {
        if let Some(ref mut chip) = self.chips[idx] {
            if !chip.is_dama {
                if chip.player == Player::Player1 && row == 7 {
                    chip.is_dama = true;
                    self.dama_pieces |= 1u64 << idx;
                } else if chip.player == Player::Player2 && row == 0 {
                    chip.is_dama = true;
                    self.dama_pieces |= 1u64 << idx;
                }
            }
        }
    }

    fn move_chip_data(&mut self, from_idx: usize, to_idx: usize) {
        let chip = self.chips[from_idx].take();
        self.chips[to_idx] = chip;

        let from_bit = 1u64 << from_idx;
        let to_bit = 1u64 << to_idx;

        if (self.p1_pieces & from_bit) != 0 {
            self.p1_pieces &= !from_bit;
            self.p1_pieces |= to_bit;
        } else {
            self.p2_pieces &= !from_bit;
            self.p2_pieces |= to_bit;
        }

        if (self.dama_pieces & from_bit) != 0 {
            self.dama_pieces &= !from_bit;
            self.dama_pieces |= to_bit;
        }
    }

    fn switch_turn(&mut self) {
        self.current_turn = match self.current_turn {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        };
    }

    fn clear_bit(&mut self, idx: usize, player: Player) {
        let bit = 1u64 << idx;
        if player == Player::Player1 {
            self.p1_pieces &= !bit;
        } else {
            self.p2_pieces &= !bit;
        }
        self.dama_pieces &= !bit;
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

    fn init_operators(&mut self) {
        for row in 0..8 {
            for col in 0..8 {
                let idx = row * 8 + col;
                let op = match row {
                    0 | 4 => match col {
                        1 => Operator::Add,
                        3 => Operator::Sub,
                        5 => Operator::Div,
                        7 => Operator::Mul,
                        _ => Operator::Add,
                    },
                    1 | 5 => match col {
                        0 => Operator::Sub,
                        2 => Operator::Add,
                        4 => Operator::Mul,
                        6 => Operator::Div,
                        _ => Operator::Add,
                    },
                    2 | 6 => match col {
                        1 => Operator::Div,
                        3 => Operator::Mul,
                        5 => Operator::Add,
                        7 => Operator::Sub,
                        _ => Operator::Add,
                    },
                    3 | 7 => match col {
                        0 => Operator::Mul,
                        2 => Operator::Div,
                        4 => Operator::Sub,
                        6 => Operator::Add,
                        _ => Operator::Add,
                    },
                    _ => Operator::Add,
                };
                self.operators[idx] = op;
            }
        }
    }
}
