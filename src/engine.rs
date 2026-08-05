use crate::board::{Board, Move};
use crate::piece::Player;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------
// Evaluation weights (HCE) — score is intentionally dominant since it's
// the actual win condition; the rest are tie-breaking heuristics.
// ---------------------------------------------------------------------

const SCORE_WEIGHT: i32 = 100;
const MATERIAL_WEIGHT: i32 = 1;
const MOBILITY_WEIGHT: i32 = 3;
const DAMA_WEIGHT: i32 = 15;

const MATE_VALUE: i32 = 1_000_000;
const INF: i32 = i32::MAX - 1;

// ---------------------------------------------------------------------
// Transposition table
// ---------------------------------------------------------------------

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

// ---------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------

pub struct Search {
    tt: HashMap<u64, TTEntry>,
    nodes: u64,
    start_time: Instant,
    time_limit: Duration,
    stop: bool,
}

impl Search {
    pub fn new() -> Self {
        Self {
            tt: HashMap::new(),
            nodes: 0,
            start_time: Instant::now(),
            time_limit: Duration::from_millis(0),
            stop: false,
        }
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
        self.tt.clear();

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
        let moves = board.generate_moves();
        if moves.is_empty() {
            return None;
        }

        let hash = hash_board(board);
        let ordered = self.order_moves(board, moves, hash);

        let mut alpha = -INF;
        let beta = INF;
        let mut best_move = None;
        let mut best_score = -INF;

        for mv in ordered {
            if let Ok(undo) = board.make_move(mv.from_row, mv.from_col, mv.to_row, mv.to_col) {
                let score = if undo.turn_switched {
                    -self.negamax(board, depth - 1, -beta, -alpha, 1)
                } else {
                    self.negamax(board, depth, alpha, beta, 1)
                };
                board.unmake_move(undo);

                if self.stop {
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
        let hash = hash_board(board);

        if let Some(entry) = self.tt.get(&hash) {
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

        let moves = board.generate_moves();
        if moves.is_empty() {
            return -(MATE_VALUE - ply as i32);
        }

        if depth == 0 {
            return self.quiescence(board, alpha, beta);
        }

        let ordered = self.order_moves(board, moves, hash);
        let mut best_score = -INF;
        let mut best_move = None;

        for mv in ordered {
            if let Ok(undo) = board.make_move(mv.from_row, mv.from_col, mv.to_row, mv.to_col) {
                let score = if undo.turn_switched {
                    -self.negamax(board, depth - 1, -beta, -alpha, ply + 1)
                } else {
                    self.negamax(board, depth, alpha, beta, ply + 1)
                };
                board.unmake_move(undo);

                if self.stop {
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
                    break;
                }
            }
        }

        let bound = if best_score <= alpha_orig {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        self.tt.insert(
            hash,
            TTEntry {
                depth,
                score: best_score,
                bound,
                best_move,
            },
        );

        best_score
    }

    fn quiescence(&mut self, board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
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

        let moves = board.generate_moves();
        if moves.is_empty() {
            return evaluate(board);
        }

        let mut best = -INF;
        for mv in moves {
            if let Ok(undo) = board.make_move(mv.from_row, mv.from_col, mv.to_row, mv.to_col) {
                let score = if undo.turn_switched {
                    -self.quiescence(board, -beta, -alpha)
                } else {
                    self.quiescence(board, alpha, beta)
                };
                board.unmake_move(undo);

                if self.stop {
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
        best
    }

    fn order_moves(&self, board: &Board, mut moves: Vec<Move>, hash: u64) -> Vec<Move> {
        let tt_best = self.tt.get(&hash).and_then(|e| e.best_move);

        moves.sort_by_key(|mv| {
            let mut key = 0i32;
            if let Some(best) = tt_best {
                if *mv == best {
                    key -= 10_000_000;
                }
            }
            if mv.is_capture() {
                key -= 1_000_000 + estimate_capture_value(board, mv);
            }
            key
        });

        moves
    }
}

/// Cheap heuristic used only for move ordering, not applied to the board:
/// finds the jumped piece along the diagonal and estimates the resulting
/// score using the landing square's operator. Lives in the engine (not
/// board.rs) because it's a search heuristic, not a game rule.
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

// ---------------------------------------------------------------------
// Evaluation (HCE)
// ---------------------------------------------------------------------

pub fn evaluate(board: &Board) -> i32 {
    let score_diff = board.p1_score - board.p2_score;

    let mut material = 0i32;
    let mut dama_diff = 0i32;

    for idx in 0..64 {
        if let Some(chip) = board.chips[idx] {
            let bit = 1u64 << idx;
            let is_p1 = (board.p1_pieces & bit) != 0;
            let sign = if is_p1 { 1 } else { -1 };
            material += sign * chip.value;
            if (board.dama_pieces & bit) != 0 {
                dama_diff += sign;
            }
        }
    }

    // Mobility uses generate_moves() from board.rs's perspective, so
    // temporarily flip current_turn is NOT an option here (no &mut).
    // Instead we approximate via a lightweight clone of just what we
    // need — see note below on this simplification.
    let p1_moves = board.generate_moves_for(Player::Player1);
    let p2_moves = board.generate_moves_for(Player::Player2);
    let mobility_diff = p1_moves as i32 - p2_moves as i32;

    let raw = SCORE_WEIGHT * score_diff
        + MATERIAL_WEIGHT * material
        + MOBILITY_WEIGHT * mobility_diff
        + DAMA_WEIGHT * dama_diff;

    match board.current_turn {
        Player::Player1 => raw,
        Player::Player2 => -raw,
    }
}

fn hash_board(board: &Board) -> u64 {
    let mut hasher = DefaultHasher::new();
    board.p1_pieces.hash(&mut hasher);
    board.p2_pieces.hash(&mut hasher);
    board.dama_pieces.hash(&mut hasher);

    let turn_code: u8 = match board.current_turn {
        Player::Player1 => 0,
        Player::Player2 => 1,
    };
    turn_code.hash(&mut hasher);

    for idx in 0..64u8 {
        if let Some(chip) = board.chips[idx as usize] {
            (idx, chip.value).hash(&mut hasher);
        }
    }

    board.forced_piece.hash(&mut hasher);
    hasher.finish()
}
