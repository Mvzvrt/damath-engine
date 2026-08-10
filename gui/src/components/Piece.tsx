import React from 'react';
import { motion } from 'framer-motion';
import { Crown } from 'lucide-react';

interface PieceProps {
  value: number;
  player: number;
  isDama: boolean;
  id: string;
}

export const Piece: React.FC<PieceProps> = ({ value, player, isDama, id }) => {
  const isP1 = player === 1;

  return (
    <motion.div
      layoutId={id}
      transition={{ type: 'spring', stiffness: 400, damping: 30 }}
      className={`relative w-12 h-12 sm:w-16 sm:h-16 rounded-full flex items-center justify-center shadow-lg font-bold font-mono text-sm sm:text-base border-2 z-10 pointer-events-none ${
        isP1
          ? 'bg-gradient-to-br from-indigo-500 to-indigo-700 text-white border-indigo-300 shadow-indigo-900/50'
          : 'bg-gradient-to-br from-rose-500 to-rose-700 text-white border-rose-300 shadow-rose-900/50'
      }`}
    >
      <span className="drop-shadow-md">{value}</span>

      {isDama && (
        <div className="absolute -top-1 -right-1 bg-amber-400 text-slate-950 p-1 rounded-full border border-amber-200 shadow-sm">
          <Crown className="w-3 h-3 fill-amber-400" />
        </div>
      )}
    </motion.div>
  );
};