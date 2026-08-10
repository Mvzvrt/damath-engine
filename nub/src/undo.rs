use crate::piece::{Chip, Player};

#[derive(Copy, Clone, Debug)]
pub struct Undo {
    pub from_idx: usize,
    pub to_idx: usize,
    pub moved_chip: Chip,
    pub promoted: bool,
    pub capture: Option<CaptureUndo>,
    pub prev_forced_piece: Option<(usize, usize)>,
    pub mover: Player,
    pub turn_switched: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct CaptureUndo {
    pub jumped_idx: usize,
    pub jumped_chip: Chip,
    pub jumped_was_dama: bool,
    pub jumped_player: Player,
    pub points_awarded: i32,
}
