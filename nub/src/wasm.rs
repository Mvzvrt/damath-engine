use crate::board::{Board, GameOutcome};
use crate::engine::Search;
use crate::operator::Operator;
use crate::piece::Player;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[derive(Serialize, Deserialize)]
pub struct JsMove {
    pub from_row: i32,
    pub from_col: i32,
    pub to_row: i32,
    pub to_col: i32,
}

#[derive(Serialize, Deserialize)]
pub struct JsBestMove {
    pub mv: JsMove,
    pub score: i32,
}

#[derive(Serialize, Deserialize)]
pub struct JsSquareInfo {
    pub index: usize,
    pub row: i32,
    pub col: i32,
    pub operator: Option<String>,
    pub chip_value: Option<i32>,
    pub chip_player: Option<u8>,
    pub is_dama: bool,
}

#[derive(Serialize, Deserialize)]
pub struct JsBoardState {
    pub squares: Vec<JsSquareInfo>,
    pub p1_score: i32,
    pub p2_score: i32,
    pub current_turn: u8,
    pub forced_piece: Option<(usize, usize)>,
    pub is_game_over: bool,
    pub outcome: Option<String>,
    pub p1_final_score: i32,
    pub p2_final_score: i32,
}

#[wasm_bindgen]
pub struct DamathWasmEngine {
    board: Board,
    search: Search,
}

#[wasm_bindgen]
impl DamathWasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            search: Search::new(),
        }
    }

    pub fn get_state(&self) -> Result<JsValue, JsValue> {
        let state = self.build_js_state();
        to_js_value(&state)
    }

    pub fn make_move(
        &mut self,
        from_row: i32,
        from_col: i32,
        to_row: i32,
        to_col: i32,
    ) -> Result<JsValue, JsValue> {
        match self.board.make_move(from_row, from_col, to_row, to_col) {
            Ok(_) => self.get_state(),
            Err(err) => Err(JsValue::from_str(err)),
        }
    }

    pub fn get_legal_moves(&self) -> Result<JsValue, JsValue> {
        let mut moves = Vec::new();
        self.board.generate_moves_into(&mut moves);
        let js_moves: Vec<JsMove> = moves
            .into_iter()
            .map(|m| JsMove {
                from_row: m.from_row,
                from_col: m.from_col,
                to_row: m.to_row,
                to_col: m.to_col,
            })
            .collect();
        serde_wasm_bindgen::to_value(&js_moves).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn find_best_move(&mut self, depth: u32, time_limit_ms: u64) -> Result<JsValue, JsValue> {
        match self
            .search
            .find_best_move(&mut self.board, depth, time_limit_ms, false)
        {
            Some((mv, score)) => {
                let res = JsBestMove {
                    mv: JsMove {
                        from_row: mv.from_row,
                        from_col: mv.from_col,
                        to_row: mv.to_row,
                        to_col: mv.to_col,
                    },
                    score,
                };
                serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            None => Ok(JsValue::NULL),
        }
    }

    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.board = Board::new();
        self.search = Search::new();
        self.get_state()
    }

    fn build_js_state(&self) -> JsBoardState {
        let mut squares = Vec::with_capacity(64);
        for idx in 0..64 {
            let row = (idx / 8) as i32;
            let col = (idx % 8) as i32;
            let bit = 1u64 << idx;

            let op_str = self.board.operators.get(idx).and_then(|op| *op).map(|op| {
                match op {
                    Operator::Add => "+",
                    Operator::Sub => "-",
                    Operator::Mul => "x",
                    Operator::Div => "÷",
                }
                .to_string()
            });

            let chip = self.board.chips.get(idx).and_then(|c| *c);
            let is_p1 = (self.board.p1_pieces & bit) != 0;
            let is_p2 = (self.board.p2_pieces & bit) != 0;
            let chip_player = if is_p1 {
                Some(1)
            } else if is_p2 {
                Some(2)
            } else {
                None
            };
            let is_dama = (self.board.dama_pieces & bit) != 0;

            squares.push(JsSquareInfo {
                index: idx,
                row,
                col,
                operator: op_str,
                chip_value: chip.map(|c| c.value),
                chip_player,
                is_dama,
            });
        }

        let outcome = self.board.terminal_outcome();
        let is_game_over = outcome.is_some();
        let outcome_str = outcome.map(|o| match o {
            GameOutcome::Player1Win => "Player1Win".to_string(),
            GameOutcome::Player2Win => "Player2Win".to_string(),
            GameOutcome::Draw => "Draw".to_string(),
        });

        let p1_final = self.board.p1_score + self.board.remaining_piece_value(Player::Player1);
        let p2_final = self.board.p2_score + self.board.remaining_piece_value(Player::Player2);

        JsBoardState {
            squares,
            p1_score: self.board.p1_score,
            p2_score: self.board.p2_score,
            current_turn: match self.board.current_turn {
                Player::Player1 => 1,
                Player::Player2 => 2,
            },
            forced_piece: self.board.forced_piece,
            is_game_over,
            outcome: outcome_str,
            p1_final_score: p1_final,
            p2_final_score: p2_final,
        }
    }
}

fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = Serializer::new().serialize_missing_as_null(true);
    value
        .serialize(&serializer)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
