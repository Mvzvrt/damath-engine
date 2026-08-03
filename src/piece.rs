#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Player {
    Player1,
    Player2,
}

#[derive(Copy, Clone, Debug)]
pub struct Chip {
    pub player: Player,
    pub value: i32,
    pub is_dama: bool,
}

impl Chip {
    pub fn new(player: Player, value: i32) -> Self {
        Self {
            player,
            value,
            is_dama: false,
        }
    }
}
