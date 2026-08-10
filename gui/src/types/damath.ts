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

export interface EngineSnapshot {
  state: JsBoardState;
  legalMoves: JsMove[];
}

export type EngineRequest =
  | { type: 'INIT' }
  | { type: 'GET_SNAPSHOT'; requestId: number }
  | { type: 'MAKE_MOVE'; requestId: number; payload: JsMove }
  | { type: 'FIND_BEST_MOVE'; requestId: number; payload: { depth: number; timeLimitMs: number } }
  | { type: 'MAKE_BEST_MOVE'; requestId: number; payload: { depth: number; timeLimitMs: number } }
  | { type: 'RESET'; requestId: number };

export type EngineResponse =
  | { type: 'READY'; payload: EngineSnapshot }
  | { type: 'RESULT'; requestId: number; payload: EngineSnapshot }
  | { type: 'BEST_MOVE_RESULT'; requestId: number; payload: JsBestMove | null }
  | { type: 'ERROR'; requestId?: number; payload: string };
