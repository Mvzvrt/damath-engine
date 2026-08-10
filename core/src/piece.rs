#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    #[inline(always)]
    pub fn opponent(self) -> Player {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Chip {
    pub value: i32,
}

impl Chip {
    #[inline(always)]
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}
