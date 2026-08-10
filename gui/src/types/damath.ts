export interface JsMove {
  from_row: number;
  from_col: number;
  to_row: number;
  to_col: number;
}

export interface JsBestMove {
  mv: JsMove;
  score: number;
}

export interface JsSquareInfo {
  index: number;
  row: number;
  col: number;
  operator: string | null;
  chip_value: number | null;
  chip_player: number | null;
  is_dama: boolean;
}

export interface JsBoardState {
  squares: JsSquareInfo[];
  p1_score: number;
  p2_score: number;
  current_turn: number;
  forced_piece: [number, number] | null;
  is_game_over: boolean;
  outcome: string | null;
  p1_final_score: number;
  p2_final_score: number;
}