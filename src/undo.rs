use crate::piece::{Chip, Player};

#[derive(Copy, Clone, Debug)]
pub struct Undo {
    pub(crate) from_idx: usize,
    pub(crate) to_idx: usize,
    pub(crate) moved_chip: Chip,
    /// Whether the moved piece was already dama *before* this move.
    pub(crate) was_dama_before: bool,
    /// Whether this move caused a promotion (needs to be un-set on undo).
    pub(crate) promoted: bool,
    pub(crate) capture: Option<CaptureUndo>,
    pub(crate) prev_forced_piece: Option<(usize, usize)>,
    pub(crate) mover: Player,
    /// Whether current_turn was flipped by this move (false while chaining).
    pub(crate) turn_switched: bool,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct CaptureUndo {
    pub(crate) jumped_idx: usize,
    pub(crate) jumped_chip: Chip,
    pub(crate) jumped_was_dama: bool,
    pub(crate) jumped_player: Player,
    pub(crate) points_awarded: i32,
}
