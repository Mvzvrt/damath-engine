import React from 'react';
import { motion } from 'framer-motion';
import { Crown } from 'lucide-react';

interface PieceProps {
  value: number;
  player: number;
  isDama: boolean;
  id: string;
  isError?: boolean;
  isGameOver?: boolean;
  staggerIndex?: number;
}

export const Piece: React.FC<PieceProps> = ({
  value,
  player,
  isDama,
  id,
  isError = false,
  isGameOver = false,
  staggerIndex = 0,
}) => {
  const isP1 = player === 1;

  return (
    <motion.div
      layoutId={id}
      animate={
        isGameOver
          ? { scale: [1, 1.25, 0], opacity: [1, 1, 0] }
          : isError
          ? { x: [-6, 6, -6, 6, 0] }
          : { scale: 1, opacity: 1 }
      }
      transition={{
        type: isGameOver ? 'keyframes' : 'spring',
        duration: isGameOver ? 0.5 : 0.3,
        delay: isGameOver ? staggerIndex * 0.07 : 0,
        stiffness: 400,
        damping: 30,
      }}
      style={{
        backgroundImage: isP1
          ? 'radial-gradient(circle at 32% 28%, #79A7CC 0%, #3E6B99 55%, #2C4C6E 100%)'
          : 'radial-gradient(circle at 32% 28%, #D9A16C 0%, #A9642F 55%, #7C4A22 100%)',
        boxShadow: isError
          ? '0 0 0 2px #F4EFDD, 0 3px 6px rgba(0,0,0,0.45)'
          : 'inset 0 1px 1px rgba(255,255,255,0.35), inset 0 -3px 4px rgba(0,0,0,0.3), 0 3px 6px rgba(0,0,0,0.45)',
      }}
      className="relative w-[80%] h-[80%] max-w-[56px] max-h-[56px] rounded-full flex items-center justify-center font-bold font-mono text-xs sm:text-sm md:text-base border border-black/20 z-10 pointer-events-none select-none text-[#F4EFDD]"
    >
      <span className="drop-shadow-md">{value}</span>

      {isDama && (
        <div className="absolute -top-1 -right-1 bg-[#F4EFDD] text-[#3B2A1D] p-0.5 rounded-full border border-[#3B2A1D]/20 shadow-sm">
          <Crown className="w-3 h-3 fill-[#3B2A1D]" />
        </div>
      )}
    </motion.div>
  );
};