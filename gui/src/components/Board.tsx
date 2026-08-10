import React, { useEffect, useMemo, useState } from 'react';
import { Square } from './Square';
import { Piece } from './Piece';
import { JsBoardState, JsMove } from '../types/damath';

interface BoardProps {
  boardState: JsBoardState;
  legalMoves: JsMove[];
  onExecuteMove: (move: JsMove) => void;
  inputDisabled?: boolean;
}

export const Board: React.FC<BoardProps> = ({
  boardState,
  legalMoves,
  onExecuteMove,
  inputDisabled = false,
}) => {
  const [selectedPos, setSelectedPos] = useState<[number, number] | null>(null);

  /*
   * Selection is UI state only.
   *
   * Rust owns legality. The legal-move list is consumed here as a
   * presentation hint for highlighting available destinations and for
   * making a sensible source-square selection.
   */
  useEffect(() => {
    if (boardState.forced_piece) {
      setSelectedPos([
        boardState.forced_piece[0],
        boardState.forced_piece[1],
      ]);
    } else {
      setSelectedPos(null);
    }
  }, [boardState.forced_piece, boardState.current_turn]);

  const availableDestinations = useMemo(
    () =>
      selectedPos
        ? legalMoves.filter(
            (move) =>
              move.from_row === selectedPos[0] &&
              move.from_col === selectedPos[1],
          )
        : [],
    [legalMoves, selectedPos],
  );

  const displaySquares = useMemo(
    () =>
      [...boardState.squares].sort((a, b) => {
        if (a.row !== b.row) return b.row - a.row;
        return a.col - b.col;
      }),
    [boardState.squares],
  );

  const hasLegalMoveFrom = (row: number, col: number) =>
    legalMoves.some(
      (move) => move.from_row === row && move.from_col === col,
    );

  const handleSquareClick = (row: number, col: number) => {
    if (inputDisabled || boardState.is_game_over) {
      return;
    }

    if (selectedPos) {
  const isSameSquare = selectedPos[0] === row && selectedPos[1] === col;

  if (!isSameSquare && hasLegalMoveFrom(row, col)) {
    setSelectedPos([row, col]);
    return;
  }

  onExecuteMove({
    from_row: selectedPos[0],
    from_col: selectedPos[1],
    to_row: row,
    to_col: col,
  });
  return;
}

    if (hasLegalMoveFrom(row, col)) {
      setSelectedPos([row, col]);
    }
  };

  return (
    <div className="p-3 bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl inline-block">
      <div className="damath-board-grid grid grid-cols-8 gap-0 border border-slate-700/60 rounded-lg overflow-hidden">
        {displaySquares.map((sq) => {
          const isSelected =
            selectedPos !== null &&
            selectedPos[0] === sq.row &&
            selectedPos[1] === sq.col;

          const isLegalTarget = availableDestinations.some(
            (move) =>
              move.to_row === sq.row &&
              move.to_col === sq.col,
          );

          const isForced =
            boardState.forced_piece != null &&
            boardState.forced_piece[0] === sq.row &&
            boardState.forced_piece[1] === sq.col;

          const hasPiece =
            sq.chip_player !== null &&
            sq.chip_value !== null;

          return (
            <Square
              key={`${sq.row}-${sq.col}`}
              row={sq.row}
              col={sq.col}
              operator={sq.operator}
              isSelected={isSelected}
              isLegalTarget={isLegalTarget}
              isForced={isForced}
              onClick={() => handleSquareClick(sq.row, sq.col)}
            >
              {hasPiece && (
                <Piece
                  id={`piece-r${sq.row}-c${sq.col}`}
                  value={sq.chip_value!}
                  player={sq.chip_player!}
                  isDama={sq.is_dama}
                />
              )}
            </Square>
          );
        })}
      </div>
    </div>
  );
};
