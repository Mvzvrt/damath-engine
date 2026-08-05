use crate::piece::Player;
use std::sync::OnceLock;

const VALUE_OFFSET: i32 = 64;
const VALUE_COUNT: usize = 129; // covers value range -64..=64

pub struct ZobristTables {
    piece_keys: Vec<u64>,
    pub side_key: u64,
    forced_keys: [u64; 64],
}

static TABLES: OnceLock<ZobristTables> = OnceLock::new();

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

pub fn tables() -> &'static ZobristTables {
    TABLES.get_or_init(|| {
        let mut seed: u64 = 0x5EED_1234_ABCD_EF01;

        let mut piece_keys = vec![0u64; 64 * 2 * 2 * VALUE_COUNT];
        for k in piece_keys.iter_mut() {
            *k = splitmix64(&mut seed);
        }

        let side_key = splitmix64(&mut seed);

        let mut forced_keys = [0u64; 64];
        for k in forced_keys.iter_mut() {
            *k = splitmix64(&mut seed);
        }

        ZobristTables {
            piece_keys,
            side_key,
            forced_keys,
        }
    })
}

impl ZobristTables {
    #[inline(always)]
    pub fn piece_key(&self, square: usize, player: Player, is_dama: bool, value: i32) -> u64 {
        let p = match player {
            Player::Player1 => 0usize,
            Player::Player2 => 1usize,
        };
        let d = if is_dama { 1usize } else { 0usize };
        let v = (value + VALUE_OFFSET).clamp(0, (VALUE_COUNT - 1) as i32) as usize;
        let idx = ((square * 2 + p) * 2 + d) * VALUE_COUNT + v;
        self.piece_keys[idx]
    }

    #[inline(always)]
    pub fn forced_key(&self, square: usize) -> u64 {
        self.forced_keys[square]
    }
}