import React from 'react';

interface SquareProps {
  row: number;
  col: number;
  operator: string | null;
  isSelected: boolean;
  isLegalTarget: boolean;
  isForced: boolean;
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
  onClick,
  children,
}) => {
  const isDark = (row + col) % 2 === 1;

  return (
    <div
      onClick={onClick}
      className={`relative w-16 h-16 sm:w-20 sm:h-20 flex items-center justify-center select-none transition-colors cursor-pointer ${
        isDark ? 'bg-slate-800' : 'bg-slate-700/40'
      } ${isSelected ? 'ring-4 ring-indigo-500 ring-inset z-10' : ''} ${
        isForced ? 'ring-4 ring-rose-500/80 ring-inset animate-pulse z-10' : ''
      }`}
    >
      {/* Operator watermark inside playable dark squares */}
      {isDark && operator && (
        <span className="absolute bottom-1 right-1.5 text-xs font-mono font-bold text-slate-500/50 pointer-events-none">
          {operator}
        </span>
      )}

      {/* Row labels on the left edge */}
      {col === 0 && (
        <span className="absolute top-1 left-1.5 text-[10px] font-mono text-slate-500 pointer-events-none">
          {row}
        </span>
      )}

      {/* Column labels on the bottom edge (Row 0) */}
      {row === 0 && (
        <span className="absolute bottom-1 left-1.5 text-[10px] font-mono text-slate-500 pointer-events-none">
          {col}
        </span>
      )}

      {/* Legal Move Highlight Indicator */}
      {isLegalTarget && (
        <div className="absolute inset-0 flex items-center justify-center z-20 pointer-events-none">
          <div className="w-5 h-5 rounded-full bg-emerald-400/60 ring-4 ring-emerald-400/20 animate-scale-up" />
        </div>
      )}

      {children}
    </div>
  );
};