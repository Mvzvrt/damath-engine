use crate::operator::Operator;
use crate::piece::{Chip, Player};
use crate::undo::{CaptureUndo, Undo};
use crate::zobrist;

const FILE_A: u64 = 0x0101010101010101;
const FILE_H: u64 = 0x8080808080808080;

#[inline(always)]
fn shift_nw(bb: u64) -> u64 {
    (bb & !FILE_A) << 7
}
#[inline(always)]
fn shift_ne(bb: u64) -> u64 {
    (bb & !FILE_H) << 9
}
#[inline(always)]
fn shift_sw(bb: u64) -> u64 {
    (bb & !FILE_A) >> 9
}
#[inline(always)]
fn shift_se(bb: u64) -> u64 {
    (bb & !FILE_H) >> 7
}

const SHIFTS: [fn(u64) -> u64; 4] = [shift_nw, shift_ne, shift_sw, shift_se];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Move {
    pub from_row: i32,
    pub from_col: i32,
    pub to_row: i32,
    pub to_col: i32,
}

impl Move {
    pub fn is_capture(&self) -> bool {
        (self.to_row - self.from_row).abs() >= 2
    }
}

pub struct Board {
    pub p1_pieces: u64,
    pub p2_pieces: u64,
    pub p1_score: i32,
    pub p2_score: i32,
    pub dama_pieces: u64,
    pub operators: [Option<Operator>; 64],
    pub chips: [Option<Chip>; 64],
    pub current_turn: Player,
    pub forced_piece: Option<(usize, usize)>,
    pub zobrist: u64,
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
            operators: [None; 64],
            current_turn: Player::Player1,
            forced_piece: None,
            zobrist: 0,
        };

        board.init_operators();
        board.init_integer_chips();
        board.zobrist = board.compute_zobrist_from_scratch();
        board
    }

    fn compute_zobrist_from_scratch(&self) -> u64 {
        let tables = zobrist::tables();
        let mut hash = 0u64;

        for idx in 0..64 {
            if let Some(chip) = self.chips[idx] {
                let bit = 1u64 << idx;
                let is_p1 = (self.p1_pieces & bit) != 0;
                let player = if is_p1 { Player::Player1 } else { Player::Player2 };
                let is_dama = (self.dama_pieces & bit) != 0;
                hash ^= tables.piece_key(idx, player, is_dama, chip.value);
            }
        }

        if self.current_turn == Player::Player2 {
            hash ^= tables.side_key;
        }

        if let Some((r, c)) = self.forced_piece {
            hash ^= tables.forced_key(r * 8 + c);
        }

        hash
    }

    pub fn display(&self) {
        println!();

        let col_header: &str = "      0     1     2     3     4     5     6     7   ";
        let top_border: &str = "   ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐  ";
        let mid_border: &str = "   ├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤  ";
        let bottom_border: &str = "   └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘  ";
        let gap: String = " ".repeat(20);
        let board_header_padding: String = " ".repeat(25);
        let operator_header_padding: String = " ".repeat(48);

        const DARK_BG: &str = "\x1b[100m";
        const RESET: &str = "\x1b[0m";

        println!("{}{}{}{}{}", board_header_padding, "BOARD", gap, operator_header_padding, "OPERATORS");
        println!("{}{}{}", col_header, gap, col_header);
        println!("{}{}{}", top_border, gap, top_border);

        for row in (0..8).rev() {
            let mut main_line = format!(" {} │", row);
            let mut op_line = format!(" {} │", row);

            for col in 0..8 {
                let idx = row * 8 + col;
                let bit = 1u64 << idx;
                let playable = self.operators[idx].is_some();

                if !playable {
                    main_line += &format!("{}     {}│", DARK_BG, RESET);
                } else {
                    match self.chips[idx] {
                        Some(ref chip) => {
                            let symbol = if (self.p1_pieces & bit) != 0 { 'A' } else { 'B' };
                            let dama_flag = if (self.dama_pieces & bit) != 0 { 'D' } else { ' ' };
                            main_line += &format!("{}{}{:>3}│", symbol, dama_flag, chip.value);
                        }
                        None => main_line += "     │",
                    }
                }

                if !playable {
                    op_line += &format!("{}     {}│", DARK_BG, RESET);
                } else {
                    let symbol = match self.operators[idx] {
                        Some(Operator::Add) => '+',
                        Some(Operator::Sub) => '-',
                        Some(Operator::Mul) => 'x',
                        Some(Operator::Div) => '÷',
                        None => ' ',
                    };
                    op_line += &format!("  {}  │", symbol);
                }
            }

            main_line += &format!(" {}", row);
            op_line += &format!(" {}", row);

            println!("{}{}{}", main_line, gap, op_line);

            if row > 0 {
                println!("{}{}{}", mid_border, gap, mid_border);
            }
        }

        println!("{}{}{}", bottom_border, gap, bottom_border);
        println!("{}{}{}", col_header, gap, col_header);
        println!();
    }

    pub fn player_has_any_capture(&self, player: Player) -> bool {
        let (own, opp) = match player {
            Player::Player1 => (self.p1_pieces, self.p2_pieces),
            Player::Player2 => (self.p2_pieces, self.p1_pieces),
        };
        let occupied = self.p1_pieces | self.p2_pieces;
        let empty = !occupied;

        let own_simple = own & !self.dama_pieces;
        if own_simple != 0 {
            for shift in SHIFTS {
                let stepped_on_opp = shift(own_simple) & opp;
                if stepped_on_opp != 0 && (shift(stepped_on_opp) & empty) != 0 {
                    return true;
                }
            }
        }

        let mut dama_bb = own & self.dama_pieces;
        while dama_bb != 0 {
            let idx = dama_bb.trailing_zeros() as i32;
            let row = idx / 8;
            let col = idx % 8;
            if self.dama_has_capture(row, col, opp, occupied) {
                return true;
            }
            dama_bb &= dama_bb - 1;
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
        let opponent_pieces = if is_p1 { self.p2_pieces } else { self.p1_pieces };
        let occupied = self.p1_pieces | self.p2_pieces;

        if is_dama {
            self.dama_has_capture(row, col, opponent_pieces, occupied)
        } else {
            let empty = !occupied;
            for shift in SHIFTS {
                let stepped_on_opp = shift(piece_bit) & opponent_pieces;
                if stepped_on_opp != 0 && (shift(stepped_on_opp) & empty) != 0 {
                    return true;
                }
            }
            false
        }
    }

    fn dama_has_capture(&self, row: i32, col: i32, opponent_pieces: u64, occupied: u64) -> bool {
        let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

        for &(dr, dc) in &directions {
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
        }
        false
    }

    pub fn generate_moves_into(&self, buf: &mut Vec<Move>) {
        buf.clear();
        let mover = self.current_turn;

        if let Some((r, c)) = self.forced_piece {
            self.push_captures_for_piece(r as i32, c as i32, buf);
            return;
        }

        if self.player_has_any_capture(mover) {
            self.push_all_captures(mover, buf);
        } else {
            self.push_all_slides(mover, buf);
        }
    }

    /// Move *count* for `player`, independent of whose turn it actually
    /// is. Used by `evaluate()` for mobility — called at every leaf node,
    /// so this deliberately never builds a Vec at all (not even a reused
    /// one): it walks the same shape as generation but only increments a
    /// counter. Kept as separate logic from generate_* on purpose, since
    /// "count moves" and "collect moves" have different perf profiles at
    /// this call frequency; any rule change here must be mirrored above.
    pub fn generate_moves_for(&self, player: Player) -> usize {
        if self.player_has_any_capture(player) {
            self.count_all_captures(player)
        } else {
            self.count_all_slides(player)
        }
    }

    fn owner_bits(&self, player: Player) -> u64 {
        match player {
            Player::Player1 => self.p1_pieces,
            Player::Player2 => self.p2_pieces,
        }
    }

    fn push_all_captures(&self, mover: Player, buf: &mut Vec<Move>) {
        let mut bb = self.owner_bits(mover);
        while bb != 0 {
            let idx = bb.trailing_zeros() as i32;
            self.push_captures_for_piece(idx / 8, idx % 8, buf);
            bb &= bb - 1;
        }
    }

    fn push_captures_for_piece(&self, row: i32, col: i32, buf: &mut Vec<Move>) {
        let from_idx = (row * 8 + col) as usize;
        let from_bit = 1u64 << from_idx;
        let is_dama = (self.dama_pieces & from_bit) != 0;
        let is_p1 = (self.p1_pieces & from_bit) != 0;
        let opponent = if is_p1 { self.p2_pieces } else { self.p1_pieces };
        let occupied = self.p1_pieces | self.p2_pieces;

        let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

        for &(dr, dc) in &directions {
            if is_dama {
                let mut step = 1;
                let mut jumped = false;
                loop {
                    let r = row + dr * step;
                    let c = col + dc * step;
                    if !(0..8).contains(&r) || !(0..8).contains(&c) {
                        break;
                    }
                    let bit = 1u64 << (r * 8 + c) as usize;

                    if !jumped {
                        if (occupied & bit) != 0 {
                            if (opponent & bit) != 0 {
                                jumped = true;
                            } else {
                                break;
                            }
                        }
                    } else {
                        if (occupied & bit) != 0 {
                            break;
                        }
                        buf.push(Move { from_row: row, from_col: col, to_row: r, to_col: c });
                    }
                    step += 1;
                }
            } else {
                let mid_r = row + dr;
                let mid_c = col + dc;
                let to_r = row + dr * 2;
                let to_c = col + dc * 2;
                if (0..8).contains(&to_r) && (0..8).contains(&to_c) {
                    let mid_bit = 1u64 << (mid_r * 8 + mid_c) as usize;
                    let to_bit = 1u64 << (to_r * 8 + to_c) as usize;
                    if (opponent & mid_bit) != 0 && (occupied & to_bit) == 0 {
                        buf.push(Move { from_row: row, from_col: col, to_row: to_r, to_col: to_c });
                    }
                }
            }
        }
    }

    fn push_all_slides(&self, mover: Player, buf: &mut Vec<Move>) {
        let occupied = self.p1_pieces | self.p2_pieces;
        let mut bb = self.owner_bits(mover);

        while bb != 0 {
            let idx = bb.trailing_zeros() as i32;
            let row = idx / 8;
            let col = idx % 8;
            let from_bit = 1u64 << idx;
            let is_dama = (self.dama_pieces & from_bit) != 0;

            if is_dama {
                for &(dr, dc) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
                    let mut step = 1;
                    loop {
                        let r = row + dr * step;
                        let c = col + dc * step;
                        if !(0..8).contains(&r) || !(0..8).contains(&c) {
                            break;
                        }
                        if (occupied & (1u64 << (r * 8 + c) as usize)) != 0 {
                            break;
                        }
                        buf.push(Move { from_row: row, from_col: col, to_row: r, to_col: c });
                        step += 1;
                    }
                }
            } else {
                let dr = if mover == Player::Player1 { 1 } else { -1 };
                for &dc in &[-1, 1] {
                    let r = row + dr;
                    let c = col + dc;
                    if (0..8).contains(&r) && (0..8).contains(&c) {
                        if (occupied & (1u64 << (r * 8 + c) as usize)) == 0 {
                            buf.push(Move { from_row: row, from_col: col, to_row: r, to_col: c });
                        }
                    }
                }
            }
            bb &= bb - 1;
        }
    }

    fn count_all_captures(&self, mover: Player) -> usize {
        let mut count = 0usize;
        let mut bb = self.owner_bits(mover);
        while bb != 0 {
            let idx = bb.trailing_zeros() as i32;
            count += self.count_captures_for_piece(idx / 8, idx % 8);
            bb &= bb - 1;
        }
        count
    }

    fn count_captures_for_piece(&self, row: i32, col: i32) -> usize {
        let from_idx = (row * 8 + col) as usize;
        let from_bit = 1u64 << from_idx;
        let is_dama = (self.dama_pieces & from_bit) != 0;
        let is_p1 = (self.p1_pieces & from_bit) != 0;
        let opponent = if is_p1 { self.p2_pieces } else { self.p1_pieces };
        let occupied = self.p1_pieces | self.p2_pieces;

        let mut count = 0usize;
        let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

        for &(dr, dc) in &directions {
            if is_dama {
                let mut step = 1;
                let mut jumped = false;
                loop {
                    let r = row + dr * step;
                    let c = col + dc * step;
                    if !(0..8).contains(&r) || !(0..8).contains(&c) {
                        break;
                    }
                    let bit = 1u64 << (r * 8 + c) as usize;

                    if !jumped {
                        if (occupied & bit) != 0 {
                            if (opponent & bit) != 0 {
                                jumped = true;
                            } else {
                                break;
                            }
                        }
                    } else {
                        if (occupied & bit) != 0 {
                            break;
                        }
                        count += 1;
                    }
                    step += 1;
                }
            } else {
                let mid_r = row + dr;
                let mid_c = col + dc;
                let to_r = row + dr * 2;
                let to_c = col + dc * 2;
                if (0..8).contains(&to_r) && (0..8).contains(&to_c) {
                    let mid_bit = 1u64 << (mid_r * 8 + mid_c) as usize;
                    let to_bit = 1u64 << (to_r * 8 + to_c) as usize;
                    if (opponent & mid_bit) != 0 && (occupied & to_bit) == 0 {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn count_all_slides(&self, mover: Player) -> usize {
        let occupied = self.p1_pieces | self.p2_pieces;
        let mut count = 0usize;
        let mut bb = self.owner_bits(mover);

        while bb != 0 {
            let idx = bb.trailing_zeros() as i32;
            let row = idx / 8;
            let col = idx % 8;
            let from_bit = 1u64 << idx;
            let is_dama = (self.dama_pieces & from_bit) != 0;

            if is_dama {
                for &(dr, dc) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
                    let mut step = 1;
                    loop {
                        let r = row + dr * step;
                        let c = col + dc * step;
                        if !(0..8).contains(&r) || !(0..8).contains(&c) {
                            break;
                        }
                        if (occupied & (1u64 << (r * 8 + c) as usize)) != 0 {
                            break;
                        }
                        count += 1;
                        step += 1;
                    }
                }
            } else {
                let dr = if mover == Player::Player1 { 1 } else { -1 };
                for &dc in &[-1, 1] {
                    let r = row + dr;
                    let c = col + dc;
                    if (0..8).contains(&r) && (0..8).contains(&c) {
                        if (occupied & (1u64 << (r * 8 + c) as usize)) == 0 {
                            count += 1;
                        }
                    }
                }
            }
            bb &= bb - 1;
        }
        count
    }

    pub fn make_move(
        &mut self,
        from_row: i32,
        from_col: i32,
        to_row: i32,
        to_col: i32,
    ) -> Result<Undo, &'static str> {
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
        let from_bit = 1u64 << from_idx;

        let own_pieces = match self.current_turn {
            Player::Player1 => self.p1_pieces,
            Player::Player2 => self.p2_pieces,
        };

        if (own_pieces & from_bit) == 0 {
            return Err(if self.chips[from_idx].is_none() {
                "Make Move Error: No piece found at selected start square."
            } else {
                "Make Move Error: Opponents pieces cannot be moved."
            });
        }

        let chip = self.chips[from_idx].unwrap();

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
        let is_dama = (self.dama_pieces & from_bit) != 0;
        let prev_forced_piece = self.forced_piece;
        let mover = self.current_turn;

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
                let promoted = self.check_promotion(to_row, to_idx);
                self.switch_turn();
                self.set_forced_piece(None);

                return Ok(Undo {
                    from_idx,
                    to_idx,
                    moved_chip: chip,
                    
                    promoted,
                    capture: None,
                    prev_forced_piece,
                    mover,
                    turn_switched: true,
                });
            } else {
                if !self.is_path_clear(from_row, from_col, to_row, to_col, None) {
                    return Err("Make Move Error: Dama path is blocked.");
                }
                self.move_chip_data(from_idx, to_idx);
                self.switch_turn();

                return Ok(Undo {
                    from_idx,
                    to_idx,
                    moved_chip: chip,
                    
                    promoted: false,
                    capture: None,
                    prev_forced_piece,
                    mover,
                    turn_switched: true,
                });
            }
        }

        let mut found_jumped = None;
        let mut curr_r = from_row + step_r;
        let mut curr_c = from_col + step_c;

        let enemy_pieces = match self.current_turn {
            Player::Player1 => self.p2_pieces,
            Player::Player2 => self.p1_pieces,
        };
        let enemy_player = self.current_turn.opponent();

        while curr_r != to_row {
            let idx = (curr_r * 8 + curr_c) as usize;
            let bit = 1u64 << idx;
            if ((self.p1_pieces | self.p2_pieces) & bit) != 0 {
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

        let jumped_r = (jumped_idx / 8) as i32;
        let jumped_c = (jumped_idx % 8) as i32;
        if !self.is_path_clear(jumped_r, jumped_c, to_row, to_col, Some(from_idx)) {
            return Err("Jump Capture Error: Landing path after capture is blocked.");
        }

        let jumped_chip = self.chips[jumped_idx].unwrap();
        let jumped_was_dama = (self.dama_pieces & jumped_bit) != 0;
        let operator = self.operators[to_idx].expect(
            "Jump Capture Error: Capture destination must be a playable square with an operator.",
        );
        let mut points = operator.apply(chip.value, jumped_chip.value);

        if is_dama && jumped_was_dama {
            points *= 4;
        } else if is_dama || jumped_was_dama {
            points *= 2;
        }

        match self.current_turn {
            Player::Player1 => self.p1_score += points,
            Player::Player2 => self.p2_score += points,
        }

        self.zobrist ^= zobrist::tables().piece_key(
            jumped_idx,
            enemy_player,
            jumped_was_dama,
            jumped_chip.value,
        );
        self.chips[jumped_idx] = None;
        self.clear_bit(jumped_idx, enemy_player);
        self.move_chip_data(from_idx, to_idx);
        let promoted = self.check_promotion(to_row, to_idx);

        let capture = Some(CaptureUndo {
            jumped_idx,
            jumped_chip,
            jumped_was_dama,
            jumped_player: enemy_player,
            points_awarded: points,
        });

        let turn_switched = if self.piece_has_any_capture(to_row, to_col) {
            self.set_forced_piece(Some((to_row as usize, to_col as usize)));
            false
        } else {
            self.set_forced_piece(None);
            self.switch_turn();
            true
        };

        Ok(Undo {
            from_idx,
            to_idx,
            moved_chip: chip,
            
            promoted,
            capture,
            prev_forced_piece,
            mover,
            turn_switched,
        })
    }

    pub fn unmake_move(&mut self, undo: Undo) {
        let tables = zobrist::tables();

        if undo.turn_switched {
            self.current_turn = undo.mover;
            self.zobrist ^= tables.side_key;
        }
        self.set_forced_piece(undo.prev_forced_piece);

        if undo.promoted {
            let bit = 1u64 << undo.to_idx;
            let is_p1 = (self.p1_pieces & bit) != 0;
            let player = if is_p1 { Player::Player1 } else { Player::Player2 };
            let value = self.chips[undo.to_idx]
                .map(|c| c.value)
                .unwrap_or(undo.moved_chip.value);
            self.zobrist ^= tables.piece_key(undo.to_idx, player, true, value);
            self.dama_pieces &= !bit;
            self.zobrist ^= tables.piece_key(undo.to_idx, player, false, value);
        }

        self.move_chip_data(undo.to_idx, undo.from_idx);

        if let Some(cap) = undo.capture {
            let bit = 1u64 << cap.jumped_idx;
            self.chips[cap.jumped_idx] = Some(cap.jumped_chip);
            match cap.jumped_player {
                Player::Player1 => self.p1_pieces |= bit,
                Player::Player2 => self.p2_pieces |= bit,
            }
            if cap.jumped_was_dama {
                self.dama_pieces |= bit;
            }
            self.zobrist ^= tables.piece_key(
                cap.jumped_idx,
                cap.jumped_player,
                cap.jumped_was_dama,
                cap.jumped_chip.value,
            );

            match undo.mover {
                Player::Player1 => self.p1_score -= cap.points_awarded,
                Player::Player2 => self.p2_score -= cap.points_awarded,
            }
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

    fn check_promotion(&mut self, row: i32, idx: usize) -> bool {
        let chip = match self.chips[idx] {
            Some(c) => c,
            None => return false,
        };
        let bit = 1u64 << idx;
        if (self.dama_pieces & bit) != 0 {
            return false;
        }
        let is_p1 = (self.p1_pieces & bit) != 0;
        if (is_p1 && row == 7) || (!is_p1 && row == 0) {
            let player = if is_p1 { Player::Player1 } else { Player::Player2 };
            let tables = zobrist::tables();
            self.zobrist ^= tables.piece_key(idx, player, false, chip.value);
            self.dama_pieces |= bit;
            self.zobrist ^= tables.piece_key(idx, player, true, chip.value);
            return true;
        }
        false
    }

    fn move_chip_data(&mut self, from_idx: usize, to_idx: usize) {
        let chip = self.chips[from_idx]
            .take()
            .expect("move_chip_data called with empty source square");

        let from_bit = 1u64 << from_idx;
        let to_bit = 1u64 << to_idx;
        let is_p1 = (self.p1_pieces & from_bit) != 0;
        let player = if is_p1 { Player::Player1 } else { Player::Player2 };
        let was_dama = (self.dama_pieces & from_bit) != 0;

        let tables = zobrist::tables();
        self.zobrist ^= tables.piece_key(from_idx, player, was_dama, chip.value);

        self.chips[to_idx] = Some(chip);

        if is_p1 {
            self.p1_pieces &= !from_bit;
            self.p1_pieces |= to_bit;
        } else {
            self.p2_pieces &= !from_bit;
            self.p2_pieces |= to_bit;
        }

        if was_dama {
            self.dama_pieces &= !from_bit;
            self.dama_pieces |= to_bit;
        }

        self.zobrist ^= tables.piece_key(to_idx, player, was_dama, chip.value);
    }

    pub(crate) fn switch_turn(&mut self) {
        self.current_turn = self.current_turn.opponent();
        self.zobrist ^= zobrist::tables().side_key;
    }

    fn set_forced_piece(&mut self, new: Option<(usize, usize)>) {
        let tables = zobrist::tables();
        if let Some((r, c)) = self.forced_piece {
            self.zobrist ^= tables.forced_key(r * 8 + c);
        }
        self.forced_piece = new;
        if let Some((r, c)) = self.forced_piece {
            self.zobrist ^= tables.forced_key(r * 8 + c);
        }
    }

    fn clear_bit(&mut self, idx: usize, player: Player) {
        let bit = 1u64 << idx;
        match player {
            Player::Player1 => self.p1_pieces &= !bit,
            Player::Player2 => self.p2_pieces &= !bit,
        }
        self.dama_pieces &= !bit;
    }

    fn init_integer_chips(&mut self) {
        let p1_layout = [
            (0, 1, -11), (0, 3, 8), (0, 5, -5), (0, 7, 2),
            (1, 0, 0), (1, 2, -3), (1, 4, 10), (1, 6, -7),
            (2, 1, -9), (2, 3, 6), (2, 5, -1), (2, 7, 4),
        ];
        for &(row, col, val) in &p1_layout {
            self.chips[row * 8 + col] = Some(Chip::new(val));
        }

        let p2_layout = [
            (5, 0, 4), (5, 2, -1), (5, 4, 6), (5, 6, -9),
            (6, 1, -7), (6, 3, 10), (6, 5, -3), (6, 7, 0),
            (7, 0, 2), (7, 2, -5), (7, 4, 8), (7, 6, -11),
        ];
        for &(row, col, val) in &p2_layout {
            self.chips[row * 8 + col] = Some(Chip::new(val));
        }
    }

    fn init_operators(&mut self) {
        for row in 0..8 {
            for col in 0..8 {
                let idx = row * 8 + col;
                let op = match row {
                    0 | 4 => match col { 1 => Some(Operator::Add), 3 => Some(Operator::Sub), 5 => Some(Operator::Div), 7 => Some(Operator::Mul), _ => None },
                    1 | 5 => match col { 0 => Some(Operator::Sub), 2 => Some(Operator::Add), 4 => Some(Operator::Mul), 6 => Some(Operator::Div), _ => None },
                    2 | 6 => match col { 1 => Some(Operator::Div), 3 => Some(Operator::Mul), 5 => Some(Operator::Add), 7 => Some(Operator::Sub), _ => None },
                    3 | 7 => match col { 0 => Some(Operator::Mul), 2 => Some(Operator::Div), 4 => Some(Operator::Sub), 6 => Some(Operator::Add), _ => None },
                    _ => None,
                };
                self.operators[idx] = op;
            }
        }
    }
}