use crate::piece::{Chip, Player};

#[derive(Copy, Clone, Debug)]
pub struct Undo {
    pub(crate) from_idx: usize,
    pub(crate) to_idx: usize,
    pub(crate) moved_chip: Chip,
    pub(crate) promoted: bool,
    pub(crate) capture: Option<CaptureUndo>,
    pub(crate) prev_forced_piece: Option<(usize, usize)>,
    pub(crate) mover: Player,
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
