import init, { DamathWasmEngine } from '../pkg/nub';
import {
  EngineRequest,
  EngineSnapshot,
  JsBestMove,
  JsBoardState,
  JsMove,
} from '../types/damath';

let engine: DamathWasmEngine | null = null;

const postError = (requestId: number | undefined, error: unknown) => {
  const message =
    typeof error === 'string'
      ? error
      : error instanceof Error
        ? error.message
        : String(error);

  self.postMessage({
    type: 'ERROR',
    ...(requestId === undefined ? {} : { requestId }),
    payload: message,
  });
};

const getSnapshot = (): EngineSnapshot => {
  if (!engine) {
    throw new Error('Engine is not initialized.');
  }

  return {
    state: engine.get_state() as JsBoardState,
    legalMoves: engine.get_legal_moves() as JsMove[],
  };
};

self.onmessage = async (event: MessageEvent<EngineRequest>) => {
  const message = event.data;

  try {
    if (message.type === 'INIT') {
      await init();
      engine = new DamathWasmEngine();

      self.postMessage({
        type: 'READY',
        payload: getSnapshot(),
      });
      return;
    }

    if (!engine) {
      postError('requestId' in message ? message.requestId : undefined, 'Engine is not initialized.');
      return;
    }

    switch (message.type) {
      case 'GET_SNAPSHOT': {
        self.postMessage({
          type: 'RESULT',
          requestId: message.requestId,
          payload: getSnapshot(),
        });
        break;
      }

      case 'MAKE_MOVE': {
        const move = message.payload;

        // Rust remains the sole authority for whether this move is valid.
        engine.make_move(
          move.from_row,
          move.from_col,
          move.to_row,
          move.to_col,
        );

        self.postMessage({
          type: 'RESULT',
          requestId: message.requestId,
          payload: getSnapshot(),
        });
        break;
      }

      case 'FIND_BEST_MOVE': {
        const { depth, timeLimitMs } = message.payload;

        const bestMove = engine.find_best_move(
          depth,
          BigInt(timeLimitMs),
        ) as JsBestMove | null;

        self.postMessage({
          type: 'BEST_MOVE_RESULT',
          requestId: message.requestId,
          payload: bestMove,
        });
        break;
      }

      case 'MAKE_BEST_MOVE': {
        const { depth, timeLimitMs } = message.payload;

        // Keep the complete AI turn inside the same Rust/WASM engine
        // instance. No move history is reconstructed in JavaScript.
        const bestMove = engine.find_best_move(
          depth,
          BigInt(timeLimitMs),
        ) as JsBestMove | null;

        if (!bestMove?.mv) {
          self.postMessage({
            type: 'RESULT',
            requestId: message.requestId,
            payload: getSnapshot(),
          });
          break;
        }

        const move = bestMove.mv;
        engine.make_move(
          move.from_row,
          move.from_col,
          move.to_row,
          move.to_col,
        );

        self.postMessage({
          type: 'RESULT',
          requestId: message.requestId,
          payload: getSnapshot(),
        });
        break;
      }

      case 'RESET': {
        engine.reset();

        self.postMessage({
          type: 'RESULT',
          requestId: message.requestId,
          payload: getSnapshot(),
        });
        break;
      }

      default:
        break;
    }
  } catch (error) {
    postError(
      'requestId' in message ? message.requestId : undefined,
      error,
    );
  }
};
