import { useEffect, useState } from 'react';
import { Crown, RotateCcw } from 'lucide-react';
import { Board } from './components/Board';
import { useEngineWorker } from './hooks/useEngineWorker';
import { JsMove } from './types/damath';

function GithubIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg fill="currentColor" viewBox="0 0 24 24" {...props}>
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
      />
    </svg>
  );
}

export default function App() {
  const engineWorker = useEngineWorker();
  const {
    isReady,
    isCalculating,
    boardState,
    legalMoves,
    engineError,
    makeMove,
    makeBestMove,
  } = engineWorker;

  const [isVsAi, setIsVsAi] = useState(true);
  const [errorSquare, setErrorSquare] = useState<{ row: number; col: number } | null>(null);

  const triggerErrorFeedback = (move?: JsMove) => {
    if (move) {
      const m = move as any;
      const target =
        m.to?.row !== undefined
          ? { row: m.to.row, col: m.to.col }
          : m.to_row !== undefined
          ? { row: m.to_row, col: m.to_col }
          : null;
      if (target) {
        setErrorSquare(target);
      }
    }

    setTimeout(() => {
      setErrorSquare(null);
    }, 600);
  };

  useEffect(() => {
    if (engineError) {
      triggerErrorFeedback();
    }
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
        await makeBestMove(8, 1000);
      } catch (error) {
        if (!cancelled) {
          triggerErrorFeedback();
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

    // Ignore self-target moves (e.g. clicking the same piece twice)
    const m = move as any;
    const isSelfMove =
      (m.from !== undefined && m.to !== undefined && m.from === m.to) ||
      (m.from_row !== undefined &&
        m.from_row === m.to_row &&
        m.from_col === m.to_col) ||
      (m.from?.row !== undefined &&
        m.from.row === m.to?.row &&
        m.from.col === m.to?.col);

    if (isSelfMove) {
      return;
    }

    try {
      await makeMove(move);
    } catch (error) {
      triggerErrorFeedback(move);
    }
  };

  const handleNewGame = () => {
    const worker = engineWorker as unknown as Record<string, () => void>;
    if (typeof worker.resetGame === 'function') {
      worker.resetGame();
    } else if (typeof worker.reset === 'function') {
      worker.reset();
    } else if (typeof worker.restart === 'function') {
      worker.restart();
    } else {
      window.location.reload();
    }
  };

  if (!isReady || !boardState) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-slate-950 text-slate-100">
        <p className="text-slate-400 animate-pulse font-mono text-sm">
          Initializing Damath Engine...
        </p>
      </div>
    );
  }

  const isP1Turn = boardState.current_turn === 1;
  const isP2Turn = boardState.current_turn === 2;

  const isGameOver = boardState.is_game_over;
  const isP1Winner = isGameOver && boardState.outcome === 'Player1Win';
  const isP2Winner = isGameOver && boardState.outcome === 'Player2Win';

  const p1Score = isGameOver
    ? (boardState.p1_final_score ?? boardState.p1_score)
    : boardState.p1_score;

  const p2Score = isGameOver
    ? (boardState.p2_final_score ?? boardState.p2_score)
    : boardState.p2_score;

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-4 md:p-8">
      <div className="w-full max-w-5xl flex flex-col lg:flex-row items-center lg:items-stretch justify-center gap-8">
        
        {/* LEFT: Board Container */}
        <div className="w-full max-w-[560px] aspect-square flex-shrink-0 flex items-center justify-center">
          <Board
            boardState={boardState}
            legalMoves={legalMoves}
            onExecuteMove={handleExecuteMove}
            inputDisabled={Boolean(isAiTurn || isCalculating)}
            errorSquare={errorSquare}
          />
        </div>

        {/* RIGHT: Sidebar */}
        <div className="w-full max-w-md flex flex-col justify-between bg-slate-900 border border-slate-800/80 rounded-2xl p-6 shadow-2xl">
          
          {/* Header */}
          <div className="flex items-center justify-between border-b border-slate-800 pb-4">
            <div>
              <h1 className="text-2xl font-black tracking-tight text-white">
                Integer Damath
              </h1>
              <p className="text-xs text-slate-400 font-mono mt-0.5">
                {isVsAi ? 'Vs Engine (AI)' : '2-Player Local'}
              </p>
            </div>
            <button
              onClick={() => setIsVsAi((value) => !value)}
              disabled={isCalculating}
              className="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 rounded-lg text-xs font-mono transition-colors disabled:opacity-50 cursor-pointer"
            >
              Toggle Mode
            </button>
          </div>

          {/* Player Cards & Game Status */}
          <div className="flex flex-col gap-4 my-auto py-6">
            
            {/* Player 2 Score Card (Top) */}
            <div
              className={`p-4 rounded-xl border transition-all flex items-center justify-between ${
                isP2Winner
                  ? 'bg-amber-950/70 border-amber-500 ring-2 ring-amber-500/40 shadow-xl shadow-amber-950/50'
                  : isP2Turn && !isGameOver
                  ? 'bg-amber-950/40 border-amber-500/60 shadow-lg shadow-amber-950/20'
                  : 'bg-slate-950/60 border-slate-800'
              }`}
            >
              <div className="flex items-center gap-3">
                <div className="w-4 h-4 rounded-full bg-amber-500 ring-4 ring-amber-500/20 flex-shrink-0" />
                <div>
                  <div className="font-bold text-slate-200 text-sm flex items-center gap-1.5">
                    <span>{isVsAi ? 'Player 2 (AI)' : 'Player 2'}</span>
                    {isP2Winner && (
                      <Crown className="w-4 h-4 text-amber-400 fill-amber-400 inline-block" />
                    )}
                  </div>
                  {isP2Winner ? (
                    <span className="text-[11px] font-mono text-amber-400 font-bold uppercase tracking-wider">
                      Victorious
                    </span>
                  ) : isP2Turn && !isGameOver ? (
                    <span className="text-[11px] font-mono text-amber-400 animate-pulse">
                      {isCalculating ? 'Calculating...' : 'Active Turn'}
                    </span>
                  ) : null}
                </div>
              </div>
              <div
                className={`text-4xl font-black font-mono tracking-tight ${
                  isP2Winner ? 'text-amber-300 scale-105' : 'text-amber-400'
                }`}
              >
                {p2Score ?? 0}
              </div>
            </div>

            {/* Victory Status & Play Again Action */}
            <div className="text-center py-1">
              {isGameOver ? (
                <div className="flex flex-col items-center gap-3">
                  <p className="text-sm font-medium font-mono text-slate-300">
                    {boardState.outcome === 'Draw'
                      ? 'Game is over. It is a draw.'
                      : `Game is over. ${isP1Winner ? 'Player 1' : 'Player 2'} is victorious.`}
                  </p>
                  <button
                    onClick={handleNewGame}
                    className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white font-mono text-xs font-bold rounded-lg shadow-lg shadow-blue-600/30 transition-all flex items-center gap-2 cursor-pointer active:scale-95"
                  >
                    <RotateCcw className="w-3.5 h-3.5" />
                    <span>Play Again</span>
                  </button>
                </div>
              ) : (
                <span className="text-xs font-mono uppercase tracking-widest text-slate-500">
                  {isAiTurn || isCalculating
                    ? 'Engine Thinking...'
                    : isP1Turn
                    ? 'Your Turn'
                    : 'Player 2 Turn'}
                </span>
              )}
            </div>

            {/* Player 1 Score Card (Bottom) */}
            <div
              className={`p-4 rounded-xl border transition-all flex items-center justify-between ${
                isP1Winner
                  ? 'bg-blue-950/70 border-blue-500 ring-2 ring-blue-500/40 shadow-xl shadow-blue-950/50'
                  : isP1Turn && !isGameOver
                  ? 'bg-blue-950/40 border-blue-500/60 shadow-lg shadow-blue-950/20'
                  : 'bg-slate-950/60 border-slate-800'
              }`}
            >
              <div className="flex items-center gap-3">
                <div className="w-4 h-4 rounded-full bg-blue-500 ring-4 ring-blue-500/20 flex-shrink-0" />
                <div>
                  <div className="font-bold text-slate-200 text-sm flex items-center gap-1.5">
                    <span>Player 1</span>
                    {isP1Winner && (
                      <Crown className="w-4 h-4 text-amber-400 fill-amber-400 inline-block" />
                    )}
                  </div>
                  {isP1Winner ? (
                    <span className="text-[11px] font-mono text-blue-400 font-bold uppercase tracking-wider">
                      Victorious
                    </span>
                  ) : isP1Turn && !isGameOver ? (
                    <span className="text-[11px] font-mono text-blue-400">
                      Active Turn
                    </span>
                  ) : null}
                </div>
              </div>
              <div
                className={`text-4xl font-black font-mono tracking-tight ${
                  isP1Winner ? 'text-blue-300 scale-105' : 'text-blue-400'
                }`}
              >
                {p1Score ?? 0}
              </div>
            </div>

          </div>

          {/* Footer */}
          <div className="border-t border-slate-800 pt-3 text-xs text-slate-500 flex justify-between items-center font-mono">
            <a
              href="https://github.com/Mvzvrt/damath-engine"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1.5 text-slate-400 hover:text-slate-200 transition-colors"
            >
              <GithubIcon className="w-4 h-4" />
              <span>Mvzvrt</span>
            </a>
          </div>

        </div>

      </div>
    </div>
  );
}