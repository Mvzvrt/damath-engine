import { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Crown, HelpCircle, RotateCcw } from 'lucide-react';
import { Board } from './components/Board';
import { HowToPlay } from './components/HowToPlay';
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

interface DifficultyOption {
  id: 'easy' | 'balanced' | 'advanced';
  label: string;
  timeLimit: number;
  depth: number;
  timeLabel: string;
  description: string;
}

const DIFFICULTY_OPTIONS: DifficultyOption[] = [
  {
    id: 'easy',
    label: 'Quick',
    depth: 24,
    timeLimit: 1000,
    timeLabel: '~1s / move',
    description: 'Damax moves fast and plays a bit loose. Good for learning the board.',
  },
  {
    id: 'balanced',
    label: 'Balanced',
    depth: 24,
    timeLimit: 5000,
    timeLabel: '~5s / move',
    description: 'Damax takes its time. Solid play, still beatable.',
  },
  {
    id: 'advanced',
    label: 'Deep',
    depth: 24,
    timeLimit: 10000,
    timeLabel: '~10s / move',
    description: 'Damax thinks as hard as it can. Expect a real fight.',
  },
];

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

  const [selectedDifficulty, setSelectedDifficulty] = useState<DifficultyOption>(DIFFICULTY_OPTIONS[1]);
  const [pendingDifficulty, setPendingDifficulty] = useState<DifficultyOption>(DIFFICULTY_OPTIONS[1]);
  const [isModalOpen, setIsModalOpen] = useState(true);
  const [isHowToPlayOpen, setIsHowToPlayOpen] = useState(false);
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
    !isModalOpen &&
    boardState?.current_turn === 2 &&
    !boardState.is_game_over;

  useEffect(() => {
    if (!isAiTurn || isCalculating) {
      return;
    }

    let cancelled = false;

    const timer = window.setTimeout(async () => {
      try {
        await makeBestMove(selectedDifficulty.depth, selectedDifficulty.timeLimit);
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
  }, [isAiTurn, isCalculating, makeBestMove, selectedDifficulty]);

  const handleExecuteMove = async (move: JsMove) => {
    if (!isReady || !boardState || boardState.is_game_over || isCalculating || isModalOpen) {
      return;
    }

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

  const handleResetGame = () => {
    const worker = engineWorker as unknown as Record<string, () => void>;
    if (typeof worker.resetGame === 'function') {
      worker.resetGame();
    } else if (typeof worker.reset === 'function') {
      worker.reset();
    } else if (typeof worker.restart === 'function') {
      worker.restart();
    }
    setPendingDifficulty(selectedDifficulty);
    setIsModalOpen(true);
  };

  const handleStartGame = () => {
    setSelectedDifficulty(pendingDifficulty);
    setIsModalOpen(false);
  };

  if (!isReady || !boardState) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-[#161F1B] text-[#EDE6D6]">
        <p className="text-[#EDE6D6]/50 animate-pulse font-mono text-sm">
          Setting up the board…
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
    <div className="min-h-screen bg-[#161F1B] text-[#EDE6D6] flex items-center justify-center p-4 md:p-8 relative">

      {/* DIFFICULTY SELECTION MODAL */}
      {isModalOpen && (
        <div className="fixed inset-0 z-50 bg-[#0F1512]/85 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="w-full max-w-md bg-[#1E2A24] border border-[#33443B] rounded-lg p-7 shadow-2xl">

            <h2 className="font-serif text-2xl text-[#F4EFDD] mb-1">
              How long should Damax think?
            </h2>
            <p className="text-sm text-[#EDE6D6]/50 mb-8">
              Longer thinking makes for a stronger, slower opponent.
            </p>

            {/* number line */}
            <div className="relative px-2 mb-6">
              <div className="absolute left-2 right-2 top-[9px] h-px bg-[#33443B]" />
              <div className="relative flex justify-between">
                {DIFFICULTY_OPTIONS.map((option) => {
                  const isSelected = pendingDifficulty.id === option.id;
                  return (
                    <button
                      key={option.id}
                      onClick={() => setPendingDifficulty(option)}
                      className="flex flex-col items-center gap-3 cursor-pointer group"
                    >
                      <span
                        className={`block rounded-full transition-all ${
                          isSelected
                            ? 'w-[19px] h-[19px] bg-[#C98246] ring-4 ring-[#C98246]/25'
                            : 'w-[13px] h-[13px] bg-[#EDE6D6]/25 group-hover:bg-[#EDE6D6]/45'
                        }`}
                      />
                      <span
                        className={`text-sm font-medium ${
                          isSelected ? 'text-[#F4EFDD]' : 'text-[#EDE6D6]/50'
                        }`}
                      >
                        {option.label}
                      </span>
                    </button>
                  );
                })}
              </div>
            </div>

            <div className="bg-[#161F1B] border border-[#33443B]/70 rounded-md px-4 py-3 mb-7 min-h-[64px]">
              <p className="text-xs font-mono text-[#C98246] mb-1">
                {pendingDifficulty.timeLabel}
              </p>
              <p className="text-sm text-[#EDE6D6]/70 leading-relaxed">
                {pendingDifficulty.description}
              </p>
            </div>

            <button
              onClick={handleStartGame}
              className="w-full py-2.5 bg-[#3E6B99] hover:bg-[#4A7BAA] text-[#F4EFDD] font-medium text-sm rounded-md transition-colors cursor-pointer active:scale-[0.98]"
            >
              Start game
            </button>
          </div>
        </div>
      )}

      {/* MAIN LAYOUT — Scaled boundaries for 1.2x visuals at 100% display zoom */}
      <div className="w-full max-w-6xl flex flex-col lg:flex-row items-center lg:items-stretch justify-center gap-10">

        {/* LEFT: Board Container */}
        <div className="w-full max-w-[672px] aspect-square flex-shrink-0 flex items-center justify-center">
          <Board
            boardState={boardState}
            legalMoves={legalMoves}
            onExecuteMove={handleExecuteMove}
            inputDisabled={Boolean(isAiTurn || isCalculating || isModalOpen)}
            errorSquare={errorSquare}
          />
        </div>

        {/* RIGHT: Sidebar */}
        <div className="w-full max-w-[538px] flex flex-col justify-between bg-[#1E2A24] border border-[#33443B] rounded-lg p-7 shadow-2xl">

          {/* Header */}
          <div className="border-b border-[#33443B] pb-4 flex items-start justify-between gap-3">
            <div>
              <h1 className="font-serif text-3xl text-[#F4EFDD]">
                Integer Damath
              </h1>
              <p className="text-sm text-[#EDE6D6]/40 mt-1">
                Damax is inspired by pre-NNUE Stockfish architecture.
              </p>
            </div>
            <button
              onClick={() => setIsHowToPlayOpen(true)}
              aria-label="How to play"
              title="How to play"
              className="flex-shrink-0 w-8 h-8 rounded-full border border-[#33443B] text-[#EDE6D6]/50 hover:text-[#F4EFDD] hover:border-[#C98246]/60 flex items-center justify-center transition-colors cursor-pointer"
            >
              <HelpCircle className="w-4 h-4" />
            </button>
          </div>

          {/* Player Cards & Game Status */}
          <div className="flex flex-col gap-4 my-auto py-6">

            {/* Damax Score Card (Top) */}
            <div
              className={`p-5 rounded-md border flex items-center justify-between transition-colors ${
                isP2Winner
                  ? 'bg-[#3A2414] border-[#C98246]'
                  : 'bg-[#161F1B] border-[#33443B]'
              }`}
            >
              <div className="flex items-center gap-3">
                <div className="w-3 h-3 rounded-full bg-[#C98246] flex-shrink-0" />
                <div>
                  <div className="font-medium text-[#EDE6D6] text-base flex items-center gap-2">
                    <span>Damax</span>
                    <span className="text-xs font-mono px-1.5 py-0.5 rounded bg-[#33443B]/60 text-[#EDE6D6]/50">
                      {selectedDifficulty.label}
                    </span>
                    {isP2Winner && <Crown className="w-4 h-4 text-[#C98246]" />}
                  </div>
                  <div className="h-[15px] mt-0.5">
                    {isP2Turn && !isGameOver && (
                      <motion.div
                        layoutId="turn-marker"
                        className="h-[3px] w-8 rounded-full bg-[#C98246]"
                      />
                    )}
                  </div>
                </div>
              </div>
              <div className="text-4xl font-mono text-[#EDE6D6]">
                {p2Score ?? 0}
              </div>
            </div>

            {/* Status line */}
            <div className="text-center py-1 min-h-[52px] flex items-center justify-center">
              <AnimatePresence mode="wait">
                {isGameOver ? (
                  <motion.div
                    key="over"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="flex flex-col items-center gap-3"
                  >
                    <p className="text-base text-[#EDE6D6]/70">
                      {boardState.outcome === 'Draw'
                        ? "It's a draw."
                        : `${isP1Winner ? 'Player 1' : 'Damax'} wins.`}
                    </p>
                    <button
                      onClick={handleResetGame}
                      className="px-5 py-2.5 bg-[#3E6B99] hover:bg-[#4A7BAA] text-[#F4EFDD] text-xs font-medium rounded-md transition-colors flex items-center gap-2 cursor-pointer active:scale-95"
                    >
                      <RotateCcw className="w-4 h-4" />
                      <span>Play again</span>
                    </button>
                  </motion.div>
                ) : (
                  <motion.span
                    key="status"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="text-sm text-[#EDE6D6]/40"
                  >
                    {isAiTurn || isCalculating
                      ? 'Damax is thinking…'
                      : isP1Turn
                      ? 'Your turn'
                      : "Damax's turn"}
                  </motion.span>
                )}
              </AnimatePresence>
            </div>

            {/* Player 1 Score Card (Bottom) */}
            <div
              className={`p-5 rounded-md border flex items-center justify-between transition-colors ${
                isP1Winner
                  ? 'bg-[#1C2E3F] border-[#3E6B99]'
                  : 'bg-[#161F1B] border-[#33443B]'
              }`}
            >
              <div className="flex items-center gap-3">
                <div className="w-3 h-3 rounded-full bg-[#3E6B99] flex-shrink-0" />
                <div>
                  <div className="font-medium text-[#EDE6D6] text-base flex items-center gap-2">
                    <span>Player 1</span>
                    {isP1Winner && <Crown className="w-4 h-4 text-[#C98246]" />}
                  </div>
                  <div className="h-[15px] mt-0.5">
                    {isP1Turn && !isGameOver && (
                      <motion.div
                        layoutId="turn-marker"
                        className="h-[3px] w-8 rounded-full bg-[#3E6B99]"
                      />
                    )}
                  </div>
                </div>
              </div>
              <div className="text-4xl font-mono text-[#EDE6D6]">
                {p1Score ?? 0}
              </div>
            </div>

          </div>

          {/* Footer */}
          <div className="border-t border-[#33443B] pt-4 text-xs text-[#EDE6D6]/40 flex justify-between items-center">
            <button
              onClick={handleResetGame}
              className="hover:text-[#EDE6D6]/80 transition-colors flex items-center gap-1.5 cursor-pointer"
            >
              <RotateCcw className="w-3.5 h-3.5" />
              <span>Reset game</span>
            </button>

            <a
              href="https://github.com/Mvzvrt/damath-engine"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1.5 hover:text-[#EDE6D6]/80 transition-colors"
            >
              <GithubIcon className="w-4 h-4" />
              <span>Mvzvrt</span>
            </a>
          </div>

        </div>

      </div>

      <HowToPlay isOpen={isHowToPlayOpen} onClose={() => setIsHowToPlayOpen(false)} />
    </div>
  );
}