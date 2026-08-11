import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, MoveDiagonal, Swords, Crown, Trophy } from 'lucide-react';

interface HowToPlayProps {
  isOpen: boolean;
  onClose: () => void;
}

interface RuleSection {
  icon: React.ElementType;
  title: string;
  body: React.ReactNode;
}

const SECTIONS: RuleSection[] = [
  {
    icon: MoveDiagonal,
    title: 'Movement',
    body: (
      <p>
        Chips move diagonally, one square at a time, onto an open space
        directly ahead. Turns alternate between players.
      </p>
    ),
  },
  {
    icon: Swords,
    title: 'Capturing & scoring',
    body: (
      <div className="space-y-2.5">
        <p>
          A capture is forced whenever an opponent&apos;s chip sits diagonally
          adjacent with an empty square directly behind it. This can be forward or
          backward diagonally.
        </p>
        <p>
          Jump the chip and land on the square behind it to remove it from
          play. The operator printed on the landing square sets the equation:
        </p>
        <p className="font-mono text-xs text-[#C98246] bg-[#161F1B] border border-[#33443B]/70 rounded px-3 py-2">
          score = capturing chip [operator] captured chip
        </p>
        <p>
          Example: a &minus;6 chip jumps a &minus;4 chip and lands on a
          &times; square &rarr; (&minus;6) &times; (&minus;4) = +24.
        </p>
        <p>
          If the landed chip can jump again, it must. Each jump in the
          sequence is scored on its own and added to the turn&apos;s total.
        </p>
      </div>
    ),
  },
  {
    icon: Crown,
    title: 'Dama promotion',
    body: (
      <div className="space-y-2.5">
        <p>
          A chip reaching the opponent&apos;s back row is crowned a Dama and
          may then slide any distance along open diagonals, forward or
          backward.
        </p>
        <ul className="space-y-1.5 text-[#EDE6D6]/70">
          <li className="flex justify-between gap-4">
            <span>Dama captures a regular chip</span>
            <span className="font-mono text-[#C98246] whitespace-nowrap">&times;2</span>
          </li>
          <li className="flex justify-between gap-4">
            <span>Regular chip captures a Dama</span>
            <span className="font-mono text-[#C98246] whitespace-nowrap">&times;2</span>
          </li>
          <li className="flex justify-between gap-4">
            <span>Dama captures a Dama</span>
            <span className="font-mono text-[#C98246] whitespace-nowrap">&times;4</span>
          </li>
        </ul>
      </div>
    ),
  },
  {
    icon: Trophy,
    title: 'Game end',
    body: (
      <p>
        The game ends once a player has no chips left or no legal move
        remains. Each player&apos;s surviving chips are added to their score,
        and the higher total wins.
      </p>
    ),
  },
];

export const HowToPlay: React.FC<HowToPlayProps> = ({ isOpen, onClose }) => {
  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
          className="fixed inset-0 z-50 bg-[#0F1512]/85 backdrop-blur-sm flex items-center justify-center p-4"
          onClick={onClose}
        >
          <motion.div
            initial={{ opacity: 0, scale: 0.97, y: 6 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.97, y: 6 }}
            transition={{ type: 'spring', stiffness: 380, damping: 32 }}
            onClick={(e) => e.stopPropagation()}
            className="w-full max-w-lg max-h-[85vh] flex flex-col bg-[#1E2A24] border border-[#33443B] rounded-lg shadow-2xl"
          >
            {/* Header */}
            <div className="flex items-start justify-between p-7 pb-5 border-b border-[#33443B]">
              <div>
                <h2 className="font-serif text-2xl text-[#F4EFDD] mb-1">
                  How to play
                </h2>
                <p className="text-sm text-[#EDE6D6]/50">
                  The rules Damax plays by, in short.
                </p>
              </div>
              <button
                onClick={onClose}
                aria-label="Close"
                className="flex-shrink-0 p-1.5 -m-1.5 rounded-md text-[#EDE6D6]/40 hover:text-[#EDE6D6]/80 hover:bg-[#161F1B] transition-colors cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Body */}
            <div className="overflow-y-auto px-7 py-6 flex flex-col gap-6">
              {SECTIONS.map(({ icon: Icon, title, body }) => (
                <div key={title} className="flex gap-4">
                  <div className="flex-shrink-0 w-8 h-8 rounded-full bg-[#161F1B] border border-[#33443B] flex items-center justify-center mt-0.5">
                    <Icon className="w-4 h-4 text-[#C98246]" />
                  </div>
                  <div className="min-w-0">
                    <h3 className="text-sm font-medium text-[#F4EFDD] mb-1.5">
                      {title}
                    </h3>
                    <div className="text-sm text-[#EDE6D6]/70 leading-relaxed">
                      {body}
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {/* Footer */}
            <div className="p-7 pt-5 border-t border-[#33443B]">
              <button
                onClick={onClose}
                className="w-full py-2.5 bg-[#3E6B99] hover:bg-[#4A7BAA] text-[#F4EFDD] font-medium text-sm rounded-md transition-colors cursor-pointer active:scale-[0.98]"
              >
                Got it
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};