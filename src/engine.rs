use crate::board::{Board, Move};
use crate::piece::Player;
use std::mem;
use std::time::{Duration, Instant};

const SCORE_WEIGHT: i32 = 100;
const MATERIAL_WEIGHT: i32 = 1;
const MOBILITY_WEIGHT: i32 = 3;
const DAMA_WEIGHT: i32 = 15;
const ADVANCEMENT_WEIGHT: i32 = 2;
const CENTRALIZATION_WEIGHT: i32 = 1;
const TEMPO_BONUS: i32 = 5;

const MATE_VALUE: i32 = 1_000_000;
const INF: i32 = i32::MAX - 1;
const NULL_MOVE_REDUCTION: u32 = 2;

const TT_INDEX_BITS: u32 = 20;
const TT_SIZE: usize = 1 << TT_INDEX_BITS;
const TT_MASK: u64 = (TT_SIZE as u64) - 1;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Copy, Clone)]
struct TTEntry {
    depth: u32,
    score: i32,
    bound: Bound,
    best_move: Option<Move>,
}

#[derive(Copy, Clone)]
struct TTSlot {
    key: u64,
    entry: TTEntry,
}

struct TranspositionTable {
    slots: Vec<Option<TTSlot>>,
}

impl TranspositionTable {
    fn new() -> Self {
        Self {
            slots: vec![None; TT_SIZE],
        }
    }

    fn clear(&mut self) {
        self.slots.iter_mut().for_each(|s| *s = None);
    }

    fn get(&self, key: u64) -> Option<&TTEntry> {
        let idx = (key & TT_MASK) as usize;
        self.slots[idx]
            .as_ref()
            .filter(|slot| slot.key == key)
            .map(|slot| &slot.entry)
    }

    fn insert(&mut self, key: u64, entry: TTEntry) {
        let idx = (key & TT_MASK) as usize;
        let should_replace = match &self.slots[idx] {
            None => true,
            Some(slot) => slot.key == key || entry.depth >= slot.entry.depth,
        };
        if should_replace {
            self.slots[idx] = Some(TTSlot { key, entry });
        }
    }
}

pub struct Search {
    tt: TranspositionTable,
    killers: [[Option<Move>; 2]; 128],
    search_path: Vec<u64>,
    move_buffers: Vec<Vec<Move>>,
    nodes: u64,
    start_time: Instant,
    time_limit: Duration,
    stop: bool,
}

impl Search {
    pub fn new() -> Self {
        Self {
            tt: TranspositionTable::new(),
            killers: [[None; 2]; 128],
            search_path: Vec::new(),
            move_buffers: Vec::new(),
            nodes: 0,
            start_time: Instant::now(),
            time_limit: Duration::from_millis(0),
            stop: false,
        }
    }

    pub fn reset(&mut self) {
        self.tt.clear();
        self.killers = [[None; 2]; 128];
        self.search_path.clear();
    }

    fn take_buffer(&mut self, idx: usize) -> Vec<Move> {
        while self.move_buffers.len() <= idx {
            self.move_buffers.push(Vec::new());
        }
        mem::take(&mut self.move_buffers[idx])
    }

    fn return_buffer(&mut self, idx: usize, buf: Vec<Move>) {
        self.move_buffers[idx] = buf;
    }

    pub fn find_best_move(
        &mut self,
        board: &mut Board,
        max_depth: u32,
        time_limit_ms: u64,
    ) -> Option<(Move, i32)> {
        self.start_time = Instant::now();
        self.time_limit = Duration::from_millis(time_limit_ms);
        self.nodes = 0;

        let mut best_overall: Option<(Move, i32)> = None;

        for depth in 1..=max_depth {
            self.stop = false;
            let result = self.negamax_root(board, depth);

            if self.stop {
                break;
            }

            if let Some((mv, score)) = result {
                best_overall = Some((mv, score));
                println!(
                    "info depth {} score {} nodes {} time {}ms",
                    depth,
                    score,
                    self.nodes,
                    self.start_time.elapsed().as_millis()
                );
                if score.abs() >= MATE_VALUE - 1000 {
                    break;
                }
            } else {
                break;
            }
        }

        best_overall
    }

    fn negamax_root(&mut self, board: &mut Board, depth: u32) -> Option<(Move, i32)> {
        let mut moves = self.take_buffer(0);
        board.generate_moves_into(&mut moves);

        if moves.is_empty() {
            self.return_buffer(0, moves);
            return None;
        }

        self.search_path.clear();
        self.search_path.push(board.zobrist);

        let key = tt_key(board);
        self.order_moves_in_place(board, &mut moves, key, 0);

        let mut alpha = -INF;
        let beta = INF;
        let mut best_move = None;
        let mut best_score = -INF;

        for &mv in moves.iter() {
            if let Ok(undo) = board.make_move(mv.from_row, mv.from_col, mv.to_row, mv.to_col) {
                let is_repetition = self.search_path.contains(&board.zobrist);
                let score = if is_repetition {
                    0
                } else {
                    self.search_path.push(board.zobrist);
                    let s = if undo.turn_switched {
                        -self.negamax(board, depth - 1, -beta, -alpha, 1)
                    } else {
                        self.negamax(board, depth, alpha, beta, 1)
                    };
                    self.search_path.pop();
                    s
                };
                board.unmake_move(undo);

                if self.stop {
                    self.return_buffer(0, moves);
                    return best_move.map(|m| (m, best_score));
                }

                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);
                }
                if best_score > alpha {
                    alpha = best_score;
                }
            }
        }

        self.return_buffer(0, moves);
        best_move.map(|m| (m, best_score))
    }

    fn negamax(
        &mut self,
        board: &mut Board,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        ply: u32,
    ) -> i32 {
        self.nodes += 1;

        if self.nodes % 2048 == 0 && self.start_time.elapsed() >= self.time_limit {
            self.stop = true;
        }
        if self.stop {
            return 0;
        }

        let alpha_orig = alpha;
        let key = tt_key(board);

        if let Some(entry) = self.tt.get(key) {
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return entry.score,
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => beta = beta.min(entry.score),
                }
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        let ply_idx = ply as usize;
        let mut moves = self.take_buffer(ply_idx);
        board.generate_moves_into(&mut moves);

        if moves.is_empty() {
            self.return_buffer(ply_idx, moves);
            return -(MATE_VALUE - ply as i32);
        }

        if depth == 0 {
            self.return_buffer(ply_idx, moves);
            return self.quiescence(board, alpha, beta, ply + 1);
        }

        if depth >= 3
            && board.forced_piece.is_none()
            && !board.player_has_any_capture(board.current_turn)
            && beta < MATE_VALUE - 1000
            && beta > -(MATE_VALUE - 1000)
        {
            board.switch_turn();
            let null_score =
                -self.negamax(board, depth - 1 - NULL_MOVE_REDUCTION, -beta, -beta + 1, ply + 1);
            board.switch_turn();

            if self.stop {
                self.return_buffer(ply_idx, moves);
                return 0;
            }
            if null_score >= beta {
                self.return_buffer(ply_idx, moves);
                return beta;
            }
        }

        self.order_moves_in_place(board, &mut moves, key, ply);
        let mut best_score = -INF;
        let mut best_move = None;

        for &mv in moves.iter() {
            if let Ok(undo) = board.make_move(mv.from_row, mv.from_col, mv.to_row, mv.to_col) {
                let is_repetition = self.search_path.contains(&board.zobrist);
                let score = if is_repetition {
                    0
                } else {
                    self.search_path.push(board.zobrist);
                    let s = if undo.turn_switched {
                        -self.negamax(board, depth - 1, -beta, -alpha, ply + 1)
                    } else {
                        self.negamax(board, depth, alpha, beta, ply + 1)
                    };
                    self.search_path.pop();
                    s
                };
                board.unmake_move(undo);

                if self.stop {
                    self.return_buffer(ply_idx, moves);
                    return 0;
                }

                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);
                }
                if best_score > alpha {
                    alpha = best_score;
                }
                if alpha >= beta {
                    if !mv.is_capture() {
                        self.store_killer(ply, mv);
                    }
                    break;
                }
            }
        }

        self.return_buffer(ply_idx, moves);

        let bound = if best_score <= alpha_orig {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        self.tt.insert(
            key,
            TTEntry {
                depth,
                score: best_score,
                bound,
                best_move,
            },
        );

        best_score
    }

    fn quiescence(&mut self, board: &mut Board, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        self.nodes += 1;

        if self.nodes % 2048 == 0 && self.start_time.elapsed() >= self.time_limit {
            self.stop = true;
        }
        if self.stop {
            return 0;
        }

        let mover = board.current_turn;
        let must_capture = board.forced_piece.is_some() || board.player_has_any_capture(mover);

        if !must_capture {
            return evaluate(board);
        }

        let ply_idx = ply as usize;
        let mut moves = self.take_buffer(ply_idx);
        board.generate_moves_into(&mut moves);

        if moves.is_empty() {
            self.return_buffer(ply_idx, moves);
            return evaluate(board);
        }

        let mut best = -INF;
        for &mv in moves.iter() {
            if let Ok(undo) = board.make_move(mv.from_row, mv.from_col, mv.to_row, mv.to_col) {
                let score = if undo.turn_switched {
                    -self.quiescence(board, -beta, -alpha, ply + 1)
                } else {
                    self.quiescence(board, alpha, beta, ply + 1)
                };
                board.unmake_move(undo);

                if self.stop {
                    self.return_buffer(ply_idx, moves);
                    return 0;
                }

                if score > best {
                    best = score;
                }
                if best > alpha {
                    alpha = best;
                }
                if alpha >= beta {
                    break;
                }
            }
        }

        self.return_buffer(ply_idx, moves);
        best
    }

    fn store_killer(&mut self, ply: u32, mv: Move) {
        let idx = (ply as usize) % self.killers.len();
        if self.killers[idx][0] != Some(mv) {
            self.killers[idx][1] = self.killers[idx][0];
            self.killers[idx][0] = Some(mv);
        }
    }

    fn order_moves_in_place(&self, board: &Board, moves: &mut [Move], key: u64, ply: u32) {
        let tt_best = self.tt.get(key).and_then(|e| e.best_move);
        let killer_idx = (ply as usize) % self.killers.len();
        let killers = self.killers[killer_idx];

        moves.sort_by_key(|mv| {
            let mut order_key = 0i32;
            if let Some(best) = tt_best {
                if *mv == best {
                    order_key -= 10_000_000;
                }
            }
            if mv.is_capture() {
                order_key -= 1_000_000 + estimate_capture_value(board, mv);
            } else if Some(*mv) == killers[0] {
                order_key -= 500_000;
            } else if Some(*mv) == killers[1] {
                order_key -= 400_000;
            }
            order_key
        });
    }
}

fn estimate_capture_value(board: &Board, mv: &Move) -> i32 {
    let step_r = (mv.to_row - mv.from_row).signum();
    let step_c = (mv.to_col - mv.from_col).signum();
    let from_idx = (mv.from_row * 8 + mv.from_col) as usize;
    let from_value = board.chips[from_idx].map(|c| c.value).unwrap_or(0);

    let mut r = mv.from_row + step_r;
    let mut c = mv.from_col + step_c;
    while r != mv.to_row {
        let idx = (r * 8 + c) as usize;
        if let Some(chip) = board.chips[idx] {
            let to_idx = (mv.to_row * 8 + mv.to_col) as usize;
            if let Some(op) = board.operators[to_idx] {
                return op.apply(from_value, chip.value).abs();
            }
        }
        r += step_r;
        c += step_c;
    }
    0
}

pub fn evaluate(board: &Board) -> i32 {
    let score_diff = board.p1_score - board.p2_score;

    let mut material = 0i32;
    let mut dama_diff = 0i32;
    let mut advancement = 0i32;
    let mut centralization = 0i32;

    for idx in 0..64 {
        if let Some(chip) = board.chips[idx] {
            let bit = 1u64 << idx;
            let is_p1 = (board.p1_pieces & bit) != 0;
            let sign = if is_p1 { 1 } else { -1 };
            material += sign * chip.value;

            let row = (idx / 8) as i32;
            let col = (idx % 8) as i32;
            let is_dama = (board.dama_pieces & bit) != 0;

            if is_dama {
                dama_diff += sign;
                let center_dist = (row - 3).abs() + (col - 3).abs();
                centralization += sign * (6 - center_dist).max(0);
            } else {
                let progress = if is_p1 { row } else { 7 - row };
                advancement += sign * progress;
            }
        }
    }

    let p1_moves = board.generate_moves_for(Player::Player1);
    let p2_moves = board.generate_moves_for(Player::Player2);
    let mobility_diff = p1_moves as i32 - p2_moves as i32;

    let mut raw = SCORE_WEIGHT * score_diff
        + MATERIAL_WEIGHT * material
        + MOBILITY_WEIGHT * mobility_diff
        + DAMA_WEIGHT * dama_diff
        + ADVANCEMENT_WEIGHT * advancement
        + CENTRALIZATION_WEIGHT * centralization;

    raw += match board.current_turn {
        Player::Player1 => TEMPO_BONUS,
        Player::Player2 => -TEMPO_BONUS,
    };

    match board.current_turn {
        Player::Player1 => raw,
        Player::Player2 => -raw,
    }
}

fn tt_key(board: &Board) -> u64 {
    board.zobrist ^ mix_scores(board.p1_score, board.p2_score)
}

fn mix_scores(p1_score: i32, p2_score: i32) -> u64 {
    let mut x = ((p1_score as i64 as u64) << 32) ^ (p2_score as i64 as u64 & 0xFFFF_FFFF);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^= x >> 31;
    x
}