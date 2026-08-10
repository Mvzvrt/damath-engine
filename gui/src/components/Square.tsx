import React from 'react';

interface SquareProps {
  row: number;
  col: number;
  operator: string | null;
  isSelected: boolean;
  isLegalTarget: boolean;
  isForced: boolean;
  isError?: boolean;
  onClick: () => void;
  children?: React.ReactNode;
}

export const Square: React.FC<SquareProps> = ({
  row,
  col,
  operator,
  isSelected,
  isLegalTarget,
  isForced,
  isError = false,
  onClick,
  children,
}) => {
  // Playable diagonal squares always have an odd coordinate sum
  const isPlayable = (row + col) % 2 === 1;

  return (
    <div
      onClick={onClick}
      className={`relative w-full h-full flex items-center justify-center select-none transition-colors cursor-pointer overflow-hidden ${
        isError
          ? 'bg-red-600 z-30 animate-pulse'
          : isPlayable
          ? 'bg-slate-100'
          : 'bg-slate-950'
      } ${
        !isError && isSelected
          ? 'ring-4 ring-blue-500 ring-inset z-20'
          : !isError && isForced
          ? 'ring-4 ring-amber-500 ring-inset animate-pulse z-20'
          : ''
      }`}
    >
      {/* Operator watermark inside playable white squares */}
      {isPlayable && operator && (
        <span
          className={`absolute bottom-0.5 right-1 text-xs sm:text-sm font-black font-mono pointer-events-none select-none z-0 ${
            isError ? 'text-white/80' : 'text-slate-800'
          }`}
        >
          {operator}
        </span>
      )}

      {/* Row labels on left edge */}
      {col === 0 && (
        <span
          className={`absolute top-0.5 left-1 text-[9px] sm:text-[11px] font-mono font-bold pointer-events-none z-0 ${
            isError
              ? 'text-white/80'
              : isPlayable
              ? 'text-slate-400'
              : 'text-slate-600'
          }`}
        >
          {row}
        </span>
      )}

      {/* Column labels on bottom edge */}
      {row === 0 && (
        <span
          className={`absolute bottom-0.5 left-1 text-[9px] sm:text-[11px] font-mono font-bold pointer-events-none z-0 ${
            isError
              ? 'text-white/80'
              : isPlayable
              ? 'text-slate-400'
              : 'text-slate-600'
          }`}
        >
          {col}
        </span>
      )}

      {/* Legal Move Target Indicator */}
      {isLegalTarget && !isError && (
        <div className="absolute inset-0 flex items-center justify-center z-20 pointer-events-none">
          <div className="w-4 h-4 sm:w-6 sm:h-6 rounded-full bg-emerald-500/80 ring-4 ring-emerald-400/30 animate-scale-up" />
        </div>
      )}

      {children}
    </div>
  );
};