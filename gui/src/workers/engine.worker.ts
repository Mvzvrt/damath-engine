import init, { DamathWasmEngine } from 'nub';

let engine: DamathWasmEngine | null = null;

self.onmessage = async (e: MessageEvent) => {
  const { type, payload } = e.data;

  if (type === 'INIT') {
    await init();
    engine = new DamathWasmEngine();
    self.postMessage({ type: 'READY' });
  } else if (type === 'FIND_BEST_MOVE') {
    if (!engine) {
      self.postMessage({ type: 'BEST_MOVE_RESULT', payload: null });
      return;
    }

    const { depth, timeLimitMs, movesHistory } = payload;

    engine.reset();
    if (movesHistory && Array.isArray(movesHistory)) {
      for (const m of movesHistory) {
        engine.make_move(m.from_row, m.from_col, m.to_row, m.to_col);
      }
    }

    const bestMove = engine.find_best_move(depth, BigInt(timeLimitMs));
    self.postMessage({ type: 'BEST_MOVE_RESULT', payload: bestMove });
  }
};