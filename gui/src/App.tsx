import { useEffect, useState } from 'react';
import init, { DamathWasmEngine } from 'nub';
import { JsBoardState, JsBestMove } from './types/damath';
import { useEngineWorker } from './hooks/useEngineWorker';

export default function App() {
  const [initialized, setInitialized] = useState(false);
  const [boardState, setBoardState] = useState<JsBoardState | null>(null);
  const [lastCalculatedMove, setLastCalculatedMove] = useState<JsBestMove | null>(null);

  const { isReady: workerReady, isCalculating, findBestMove } = useEngineWorker();

  useEffect(() => {
    async function loadWasm() {
      await init();
      const engine = new DamathWasmEngine();
      setBoardState(engine.get_state() as JsBoardState);
      setInitialized(true);
    }
    loadWasm();
  }, []);

  const handleTestWorkerSearch = async () => {
    // Run depth 6 search with a 2000ms time limit on the Web Worker thread
    const bestMove = await findBestMove(6, 2000, []);
    setLastCalculatedMove(bestMove);
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-4 space-y-4">
      <h1 className="text-3xl font-bold tracking-tight">Integer Damath</h1>

      {initialized && boardState ? (
        <div className="bg-slate-900 border border-slate-800 p-6 rounded-xl text-center space-y-4 w-96">
          <div>
            <p className="text-emerald-400 font-medium">✓ Wasm Engine Loaded</p>
            <p className="text-xs text-slate-400">Current Turn: Player {boardState.current_turn}</p>
            <p className="text-xs text-slate-400">Worker Status: {workerReady ? 'Ready' : 'Initializing...'}</p>
          </div>

          <button
            onClick={handleTestWorkerSearch}
            disabled={!workerReady || isCalculating}
            className="w-full py-2 px-4 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-medium transition"
          >
            {isCalculating ? 'Engine Thinking...' : 'Test AI Worker Search'}
          </button>

          {lastCalculatedMove && (
            <div className="p-3 bg-slate-950 rounded-lg text-left text-xs font-mono text-emerald-300">
              <p>Best Move: ({lastCalculatedMove.mv.from_row}, {lastCalculatedMove.mv.from_col}) → ({lastCalculatedMove.mv.to_row}, {lastCalculatedMove.mv.to_col})</p>
              <p>Eval Score: {lastCalculatedMove.score}</p>
            </div>
          )}
        </div>
      ) : (
        <p className="text-slate-400 animate-pulse">Initializing WebAssembly Engine...</p>
      )}
    </div>
  );
}