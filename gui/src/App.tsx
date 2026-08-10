import { useEffect, useState } from 'react';
import init, { DamathWasmEngine } from 'nub';
import { JsBoardState } from './types/damath';

export default function App() {
  const [initialized, setInitialized] = useState(false);
  const [boardState, setBoardState] = useState<JsBoardState | null>(null);

  useEffect(() => {
    async function loadWasm() {
      await init();
      const engine = new DamathWasmEngine();
      setBoardState(engine.get_state() as JsBoardState);
      setInitialized(true);
    }
    loadWasm();
  }, []);

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-4">
      <h1 className="text-3xl font-bold mb-4 tracking-tight">Integer Damath</h1>
      {initialized && boardState ? (
        <div className="bg-slate-900 border border-slate-800 p-6 rounded-xl text-center space-y-2">
          <p className="text-emerald-400 font-medium">✓ Wasm Engine Loaded</p>
          <p className="text-sm text-slate-400">Current Turn: Player {boardState.current_turn}</p>
          <p className="text-sm text-slate-400">Squares Initialized: {boardState.squares.length}</p>
        </div>
      ) : (
        <p className="text-slate-400 animate-pulse">Initializing WebAssembly Engine...</p>
      )}
    </div>
  );
}