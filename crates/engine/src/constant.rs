use crate::{board::*, zobrist::*};
use std::{collections::HashMap, sync::LazyLock};

pub const MIN: i32 = -99999;
pub const KILL: i32 = MIN + 100;
pub const MAX: i32 = 99999;
pub const RECORD_SIZE: i32 = 0x1FFFFE;
pub const MAX_DEPTH: i32 = 64;

pub static FEN_MAP: LazyLock<HashMap<char, Chess>> = LazyLock::new(|| {
    HashMap::from([
        ('k', Chess::Black(ChessType::King)),
        ('a', Chess::Black(ChessType::Advisor)),
        ('b', Chess::Black(ChessType::Bishop)),
        ('n', Chess::Black(ChessType::Knight)),
        ('r', Chess::Black(ChessType::Rook)),
        ('c', Chess::Black(ChessType::Cannon)),
        ('p', Chess::Black(ChessType::Pawn)),
        ('K', Chess::Red(ChessType::King)),
        ('A', Chess::Red(ChessType::Advisor)),
        ('B', Chess::Red(ChessType::Bishop)),
        ('N', Chess::Red(ChessType::Knight)),
        ('R', Chess::Red(ChessType::Rook)),
        ('C', Chess::Red(ChessType::Cannon)),
        ('P', Chess::Red(ChessType::Pawn)),
    ])
});
pub static ZOBRIST_TABLE: LazyLock<Zobristable> = LazyLock::new(|| Zobristable::new());
pub static ZOBRIST_TABLE_LOCK: LazyLock<Zobristable> = LazyLock::new(|| Zobristable::new());
