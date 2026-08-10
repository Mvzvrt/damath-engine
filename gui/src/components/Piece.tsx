import React from 'react';
import { motion } from 'framer-motion';
import { Crown } from 'lucide-react';

interface PieceProps {
  value: number;
  player: number;
  isDama: boolean;
  id: string;
  isError?: boolean;
}

export const Piece: React.FC<PieceProps> = ({
  value,
  player,
  isDama,
  id,
  isError = false,
}) => {
  const isP1 = player === 1;

  return (
    <motion.div
      layoutId={id}
      animate={isError ? { x: [-6, 6, -6, 6, 0] } : {}}
      transition={{
        type: 'spring',
        stiffness: 400,
        damping: 30,
        x: { duration: 0.3 },
      }}
      className={`relative w-[80%] h-[80%] max-w-[56px] max-h-[56px] rounded-full flex items-center justify-center shadow-md font-bold font-mono text-xs sm:text-sm md:text-base border-2 z-10 pointer-events-none select-none ${
        isP1
          ? 'bg-gradient-to-br from-blue-600 to-indigo-800 text-white border-blue-200 shadow-blue-950/40'
          : 'bg-gradient-to-br from-amber-500 to-orange-600 text-slate-950 border-amber-200 shadow-amber-950/40'
      } ${isError ? 'ring-2 ring-white shadow-red-950' : ''}`}
    >
      <span className="drop-shadow-md">{value}</span>

      {isDama && (
        <div className="absolute -top-1 -right-1 bg-amber-300 text-slate-950 p-0.5 rounded-full border border-amber-100 shadow-sm">
          <Crown className="w-3 h-3 fill-slate-950" />
        </div>
      )}
    </motion.div>
  );
};