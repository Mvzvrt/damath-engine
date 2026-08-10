import { useEffect, useState } from 'react';
import { Board } from './components/Board';
import { useEngineWorker } from './hooks/useEngineWorker';
import { JsMove } from './types/damath';

export default function App() {
  const {
    isReady,
    isCalculating,
    boardState,
    legalMoves,
    engineError,
    makeMove,
    makeBestMove,
  } = useEngineWorker();

  const [isVsAi, setIsVsAi] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    setErrorMessage(engineError);
  }, [engineError]);

  const isAiTurn =
    isVsAi &&
    boardState?.current_turn === 2 &&
    !boardState.is_game_over;

  useEffect(() => {
    if (!isAiTurn || isCalculating) {
      return;
    }

    let cancelled = false;

    const timer = window.setTimeout(async () => {
      try {
        setErrorMessage(null);
        await makeBestMove(8, 1000);
      } catch (error) {
        if (!cancelled) {
          setErrorMessage(
            typeof error === 'string'
              ? error
              : error instanceof Error
                ? error.message
                : String(error),
          );
        }
      }
    }, 300);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [isAiTurn, isCalculating, makeBestMove]);

  const handleExecuteMove = async (move: JsMove) => {
    if (!isReady || !boardState || boardState.is_game_over || isCalculating) {
      return;
    }
    try {
      setErrorMessage(null);
      await makeMove(move);
    } catch (error) {
      setErrorMessage(
        typeof error === 'string'
          ? error
          : error instanceof Error
            ? error.message
            : String(error),
      );
    }
  };

  if (!isReady || !boardState) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-slate-950 text-slate-100">
        <p className="text-slate-400 animate-pulse">
          Initializing Damath Engine...
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center min-h-screen bg-slate-950 text-slate-100 p-4 space-y-4">
      <header className="text-center space-y-2">
        <h1 className="text-3xl font-bold tracking-tight">Integer Damath</h1>

        <div className="flex items-center justify-center gap-4 text-xs font-mono">
          <p>
            Turn:{' '}
            <span
              className={
                boardState.current_turn === 1
                  ? 'text-indigo-400 font-bold'
                  : 'text-rose-400 font-bold'
              }
            >
              Player {boardState.current_turn}
              {isAiTurn || isCalculating ? ' (Thinking...)' : ''}
            </span>
          </p>

          <p>
            Score P1:{' '}
            <span className="text-indigo-300">
              {boardState.p1_score}
            </span>
          </p>

          <p>
            Score P2:{' '}
            <span className="text-rose-300">
              {boardState.p2_score}
            </span>
          </p>
        </div>

        <button
          onClick={() => setIsVsAi((value) => !value)}
          disabled={isCalculating}
          className="px-3 py-1 bg-slate-800 border border-slate-700 rounded text-xs font-mono text-slate-300 hover:bg-slate-700 disabled:opacity-50"
        >
          Mode: {isVsAi ? 'Vs Engine (AI)' : '2-Player Local'}
        </button>
      </header>

      {errorMessage && (
        <div className="px-3 py-1 bg-rose-500/10 border border-rose-500/30 text-rose-400 text-xs rounded font-mono">
          {errorMessage}
        </div>
      )}

      {boardState.is_game_over && (
  <div className="px-4 py-2 bg-amber-500/10 border border-amber-500/30 text-amber-300 text-sm rounded font-mono text-center space-y-1">
    <p className="font-bold">
      {boardState.outcome === 'Draw'
        ? 'Game Over — Draw'
        : `Game Over — ${boardState.outcome === 'Player1Win' ? 'Player 1' : 'Player 2'} Wins`}
    </p>
    <p className="text-xs opacity-80">
      Final Score — P1: {boardState.p1_final_score} · P2: {boardState.p2_final_score}
    </p>
  </div>
)}

      <Board
        boardState={boardState}
        legalMoves={legalMoves}
        onExecuteMove={handleExecuteMove}
        inputDisabled={Boolean(isAiTurn || isCalculating)}
      />
    </div>
  );
}
