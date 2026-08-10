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

const WOOD_GRAIN =
  'repeating-linear-gradient(115deg, rgba(0,0,0,0.16) 0px, rgba(0,0,0,0.16) 1px, transparent 1px, transparent 5px), linear-gradient(135deg, #3B2A1D 0%, #2A1D13 60%, #33241A 100%)';
const PAPER_GRAIN =
  'radial-gradient(rgba(59,42,29,0.05) 1px, transparent 1px), linear-gradient(160deg, #EDE6D6 0%, #E6DDC8 100%)';

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
  const isPlayable = (row + col) % 2 === 1;

  return (
    <div
      onClick={onClick}
      style={
        isError
          ? undefined
          : {
              backgroundImage: isPlayable ? PAPER_GRAIN : WOOD_GRAIN,
              backgroundSize: isPlayable ? '6px 6px, 100% 100%' : '100% 100%',
            }
      }
      className={`relative w-full h-full flex items-center justify-center select-none transition-colors cursor-pointer overflow-hidden ${
        isError ? 'bg-[#8C3A2E] z-30' : ''
      } ${
        !isError && isSelected
          ? 'ring-inset ring-[3px] ring-[#F4EFDD]/90 z-20'
          : !isError && isForced
          ? 'ring-inset ring-[3px] ring-[#C98246] animate-pulse z-20'
          : ''
      }`}
    >
      {/* Operator watermark inside playable squares */}
      {isPlayable && operator && (
        <span
          className={`absolute bottom-0.5 right-1 text-xs sm:text-sm font-semibold font-mono pointer-events-none select-none z-0 ${
            isError ? 'text-white/80' : 'text-[#3B2A1D]/60'
          }`}
        >
          {operator}
        </span>
      )}

      {/* Row labels on left edge */}
      {col === 0 && (
        <span
          className={`absolute top-0.5 left-1 text-[9px] sm:text-[11px] font-mono pointer-events-none z-0 ${
            isError
              ? 'text-white/80'
              : isPlayable
              ? 'text-[#3B2A1D]/40'
              : 'text-[#EDE6D6]/25'
          }`}
        >
          {row}
        </span>
      )}

      {/* Column labels on bottom edge */}
      {row === 0 && (
        <span
          className={`absolute bottom-0.5 left-1 text-[9px] sm:text-[11px] font-mono pointer-events-none z-0 ${
            isError
              ? 'text-white/80'
              : isPlayable
              ? 'text-[#3B2A1D]/40'
              : 'text-[#EDE6D6]/25'
          }`}
        >
          {col}
        </span>
      )}

      {/* High-contrast legal target indicator */}
      {isLegalTarget && !isError && (
        <div className="absolute inset-0 flex items-center justify-center z-20 pointer-events-none">
          <div className="w-4 h-4 sm:w-6 sm:h-6 rounded-full bg-[#C98246] ring-4 ring-[#C98246]/30 shadow-md border border-[#2A1D13]/30 animate-scale-up" />
        </div>
      )}

      {children}
    </div>
  );
};