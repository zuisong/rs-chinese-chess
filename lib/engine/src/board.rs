use std::vec;

use crate::constant::{FEN_MAP, KILL, MAX, MAX_DEPTH, MIN, RECORD_SIZE, ZOBRIST_TABLE, ZOBRIST_TABLE_LOCK};

pub const BOARD_WIDTH: i32 = 9;
pub const BOARD_HEIGHT: i32 = 10;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Chess {
    Black(ChessType),
    Red(ChessType),
    None,
}

impl Chess {
    pub fn value(&self) -> i32 {
        match self.chess_type() {
            Some(ct) => ct.type_value(),
            None => 0,
        }
    }
    pub fn belong_to(&self, player: Player) -> bool {
        Some(player) == self.player()
    }
    pub fn chess_type(&self) -> Option<ChessType> {
        match self {
            Chess::Black(ct) => Some(ct.to_owned()),
            Chess::Red(ct) => Some(ct.to_owned()),
            Chess::None => None,
        }
    }
    pub fn player(&self) -> Option<Player> {
        match self {
            Chess::Black(_) => Some(Player::Black),
            Chess::Red(_) => Some(Player::Red),
            Chess::None => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChessType {
    King,    // 帅
    Advisor, // 士
    Bishop,  // 相
    Knight,  // 马
    Rook,    // 车
    Cannon,  // 炮
    Pawn,    // 兵
}

impl ChessType {
    pub fn value(&self) -> i32 {
        match self {
            ChessType::King => 1,
            ChessType::Advisor => 2,
            ChessType::Bishop => 3,
            ChessType::Knight => 4,
            ChessType::Rook => 5,
            ChessType::Cannon => 6,
            ChessType::Pawn => 0,
        }
    }
    pub fn type_value(&self) -> i32 {
        match self {
            ChessType::King => 5,
            ChessType::Advisor => 1,
            ChessType::Bishop => 1,
            ChessType::Knight => 3,
            ChessType::Rook => 4,
            ChessType::Cannon => 3,
            ChessType::Pawn => 2,
        }
    }

    pub fn move_value(&self) -> i32 {
        match self {
            ChessType::King => 1,
            ChessType::Advisor => 2,
            ChessType::Bishop => 2,
            ChessType::Knight => 5,
            ChessType::Rook => 6,
            ChessType::Cannon => 4,
            ChessType::Pawn => 3,
        }
    }

    pub fn name_value(&self) -> &'static str {
        match self {
            ChessType::King => "帅",
            ChessType::Advisor => "士",
            ChessType::Bishop => "相",
            ChessType::Knight => "马",
            ChessType::Rook => "车",
            ChessType::Cannon => "炮",
            ChessType::Pawn => "兵",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Player {
    Red,
    Black,
}

impl Player {
    pub fn value(&self) -> i32 {
        if self == &Player::Red {
            0
        } else {
            1
        }
    }
    pub fn next(&self) -> Player {
        if self == &Player::Red {
            Player::Black
        } else {
            Player::Red
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Position {
    pub row: i32,
    pub col: i32,
}

impl From<(i32, i32)> for Position {
    fn from(value: (i32, i32)) -> Self {
        Position {
            row: value.1,
            col: value.0,
        }
    }
}

impl Position {
    pub fn new(row: i32, col: i32) -> Self {
        Position { row, col }
    }
    pub fn up(&self, delta: i32) -> Self {
        Position::new(self.row - delta, self.col)
    }
    pub fn down(&self, delta: i32) -> Self {
        Position::new(self.row + delta, self.col)
    }
    pub fn left(&self, delta: i32) -> Self {
        Position::new(self.row, self.col - delta)
    }
    pub fn right(&self, delta: i32) -> Self {
        Position::new(self.row, self.col + delta)
    }
    pub fn flip(&self) -> Self {
        Position::new(BOARD_HEIGHT - 1 - self.row, BOARD_WIDTH - 1 - self.col)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Move {
    pub player: Player, // 玩家
    pub from: Position, // 起手位置
    pub to: Position,   // 落子位置
    pub chess: Chess,   // 记录一下运的子，如果后面没用到就删了
    pub capture: Chess, // 这一步吃的子
}
impl Move {
    pub fn stay() -> Move {
        Move {
            player: Player::Red,
            from: Position::new(0, 0),
            to: Position::new(0, 0),
            chess: Chess::None,
            capture: Chess::None,
        }
    }
    pub fn is_valid(&self) -> bool {
        self.chess != Chess::None && self.from != self.to
    }
    pub fn with_target(&self, to: Position, capture: Chess) -> Move {
        Move {
            player: self.player,
            from: self.from,
            to,
            chess: self.chess,
            capture,
        }
    }
}

impl From<&str> for Position {
    fn from(m: &str) -> Self {
        let mb = m.as_bytes();
        Position::new(
            BOARD_HEIGHT - 1 - (mb[1] - '0' as u8) as i32,
            (mb[0] - 'a' as u8) as i32,
        )
    }
}
impl ToString for Position {
    fn to_string(&self) -> String {
        format!(
            "{}{}",
            char::from_u32((self.col as u8 + 'a' as u8) as u32).unwrap(),
            char::from_u32(((BOARD_HEIGHT as u8 - 1 - self.row as u8) + '0' as u8) as u32).unwrap()
        )
    }
}

#[derive(Clone, Debug)]
pub struct Record {
    pub value: i32,
    pub depth: i32,
    pub best_move: Option<Move>,
    pub zobrist_lock: u64,
    pub turn: Player,
}

pub struct Board {
    // 9×10的棋盘，红方在下，黑方在上
    pub chesses: [[Chess; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    pub turn: Player,
    pub counter: i32,
    pub gen_counter: i32,
    pub move_history: Vec<Move>,
    pub best_moves_last: Vec<Move>,
    pub records: Vec<Option<Record>>,
    pub zobrist_value: u64,
    pub zobrist_value_lock: u64,
    pub distance: i32,
    pub select_pos: Position,
}

// 棋子是否在棋盘内
pub fn in_board(pos: Position) -> bool {
    pos.row >= 0 && pos.row < BOARD_HEIGHT && pos.col >= 0 && pos.col < BOARD_WIDTH
}

// 棋子是否在玩家的楚河汉界以内
pub fn in_country(row: i32, player: Player) -> bool {
    let base_row = if player == Player::Red { BOARD_HEIGHT - 1 } else { 0 };
    (row - base_row).abs() < BOARD_HEIGHT / 2
}

// 棋子是否在九宫格内
pub fn in_palace(pos: Position, player: Player) -> bool {
    if player == Player::Black {
        pos.row >= 0 && pos.row < 3 && pos.col >= 3 && pos.col < 6
    } else {
        pos.row >= 7 && pos.row < BOARD_HEIGHT && pos.col >= 3 && pos.col < 6
    }
}

const KING_VALUE_TABLE: [[i32; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 1, 1, 0, 0, 0],
    [0, 0, 0, 2, 2, 2, 0, 0, 0],
    [0, 0, 0, 11, 15, 11, 0, 0, 0],
];

const ADVISOR_VALUE_TABLE: [[i32; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 20, 0, 20, 0, 0, 0],
    [0, 0, 0, 0, 23, 0, 0, 0, 0],
    [0, 0, 0, 20, 0, 20, 0, 0, 0],
];

const BISHOP_VALUE_TABLE: [[i32; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 20, 0, 0, 0, 20, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [18, 0, 0, 0, 23, 0, 0, 0, 18],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 20, 0, 0, 0, 20, 0, 0],
];

const ROOK_VALUE_TABLE: [[i32; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [
    [206, 208, 207, 213, 214, 213, 207, 208, 206],
    [206, 212, 209, 216, 233, 216, 209, 212, 206],
    [206, 208, 207, 214, 216, 214, 207, 208, 206],
    [206, 213, 213, 216, 216, 216, 213, 213, 206],
    [208, 211, 211, 214, 215, 214, 211, 211, 208],
    [208, 212, 212, 214, 215, 214, 212, 212, 208],
    [204, 209, 204, 212, 214, 212, 204, 209, 204],
    [198, 208, 204, 212, 212, 212, 204, 208, 198],
    [200, 208, 206, 212, 200, 212, 206, 208, 200],
    [194, 206, 204, 212, 200, 212, 204, 206, 194],
];

const KNIGHT_VALUE_TABLE: [[i32; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [
    [90, 90, 90, 96, 90, 96, 90, 90, 90],
    [90, 96, 103, 97, 94, 97, 103, 96, 90],
    [92, 98, 99, 103, 99, 103, 99, 98, 92],
    [93, 108, 100, 107, 100, 107, 100, 108, 93],
    [90, 100, 99, 103, 104, 103, 99, 100, 90],
    [90, 98, 101, 102, 103, 102, 101, 98, 90],
    [92, 94, 98, 95, 98, 95, 98, 94, 92],
    [93, 92, 94, 95, 92, 95, 94, 92, 93],
    [85, 90, 92, 93, 78, 93, 92, 90, 85],
    [88, 85, 90, 88, 90, 88, 90, 85, 88],
];

const CANNON_VALUE_TABLE: [[i32; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [
    [100, 100, 96, 91, 90, 91, 96, 100, 100],
    [98, 98, 96, 92, 89, 92, 96, 98, 98],
    [97, 97, 96, 91, 92, 91, 96, 97, 97],
    [96, 99, 99, 98, 100, 98, 99, 99, 96],
    [96, 96, 96, 96, 100, 96, 96, 96, 96],
    [95, 96, 99, 96, 100, 96, 99, 96, 95],
    [96, 96, 96, 96, 96, 96, 96, 96, 96],
    [97, 96, 100, 99, 101, 99, 100, 96, 97],
    [96, 97, 98, 98, 98, 98, 98, 97, 96],
    [96, 96, 97, 99, 99, 99, 97, 96, 96],
];

const PAWN_VALUE_TABLE: [[i32; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [
    [9, 9, 9, 11, 13, 11, 9, 9, 9],
    [19, 24, 34, 42, 44, 42, 34, 24, 19],
    [19, 24, 32, 37, 37, 37, 32, 24, 19],
    [19, 23, 27, 29, 30, 29, 27, 23, 19],
    [14, 18, 20, 27, 29, 27, 20, 18, 14],
    [7, 0, 13, 0, 16, 0, 13, 0, 7],
    [7, 0, 7, 0, 15, 0, 7, 0, 7],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
];

const INITIATIVE_BONUS: i32 = 3;

const RECORD_NONE: Option<Record> = None;
impl Board {
    pub fn init() -> Self {
        let mut board = Board {
            chesses: [
                [
                    Chess::Black(ChessType::Rook),
                    Chess::Black(ChessType::Knight),
                    Chess::Black(ChessType::Bishop),
                    Chess::Black(ChessType::Advisor),
                    Chess::Black(ChessType::King),
                    Chess::Black(ChessType::Advisor),
                    Chess::Black(ChessType::Bishop),
                    Chess::Black(ChessType::Knight),
                    Chess::Black(ChessType::Rook),
                ],
                [
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                ],
                [
                    Chess::None,
                    Chess::Black(ChessType::Cannon),
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::Black(ChessType::Cannon),
                    Chess::None,
                ],
                [
                    Chess::Black(ChessType::Pawn),
                    Chess::None,
                    Chess::Black(ChessType::Pawn),
                    Chess::None,
                    Chess::Black(ChessType::Pawn),
                    Chess::None,
                    Chess::Black(ChessType::Pawn),
                    Chess::None,
                    Chess::Black(ChessType::Pawn),
                ],
                [
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                ],
                [
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                ],
                [
                    Chess::Red(ChessType::Pawn),
                    Chess::None,
                    Chess::Red(ChessType::Pawn),
                    Chess::None,
                    Chess::Red(ChessType::Pawn),
                    Chess::None,
                    Chess::Red(ChessType::Pawn),
                    Chess::None,
                    Chess::Red(ChessType::Pawn),
                ],
                [
                    Chess::None,
                    Chess::Red(ChessType::Cannon),
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::Red(ChessType::Cannon),
                    Chess::None,
                ],
                [
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                    Chess::None,
                ],
                [
                    Chess::Red(ChessType::Rook),
                    Chess::Red(ChessType::Knight),
                    Chess::Red(ChessType::Bishop),
                    Chess::Red(ChessType::Advisor),
                    Chess::Red(ChessType::King),
                    Chess::Red(ChessType::Advisor),
                    Chess::Red(ChessType::Bishop),
                    Chess::Red(ChessType::Knight),
                    Chess::Red(ChessType::Rook),
                ],
            ],
            turn: Player::Red,
            counter: 0,
            gen_counter: 0,
            move_history: vec![],
            best_moves_last: vec![],
            records: vec![],
            zobrist_value: 0,
            zobrist_value_lock: 0,
            distance: 0,
            select_pos: Position { row: 1, col: 1 },
        };
        board.zobrist_value = ZOBRIST_TABLE.calc_chesses(&board.chesses);
        board.zobrist_value_lock = ZOBRIST_TABLE_LOCK.calc_chesses(&board.chesses);
        board
    }
    pub fn empty() -> Self {
        Board {
            chesses: [[Chess::None; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
            turn: Player::Red,
            counter: 0,
            gen_counter: 0,
            move_history: vec![],
            best_moves_last: vec![],
            records: vec![],
            zobrist_value: 0,
            zobrist_value_lock: 0,
            distance: 0,
            select_pos: Position { row: 1, col: 1 },
        }
    }
    pub fn from_fen(fen: &str) -> Self {
        let mut board = Board::empty();
        let mut parts = fen.split(" ");
        let pos = parts.next().unwrap();
        let mut i = 0;
        for row in pos.split("/") {
            let mut j = 0;
            for col in row.chars() {
                if col.is_numeric() {
                    j += col.to_digit(10).unwrap() as i32;
                } else {
                    if let Some(chess) = (FEN_MAP).get(&col) {
                        board.set_chess(Position::new(i, j), chess.to_owned());
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        board.zobrist_value = ZOBRIST_TABLE.calc_chesses(&board.chesses);
        board.zobrist_value_lock = ZOBRIST_TABLE_LOCK.calc_chesses(&board.chesses);
        let turn = parts.next().unwrap();
        if turn == "b" {
            board.turn = Player::Black;
        }
        board
    }
    pub fn apply_move(&mut self, m: &Move) {
        let chess = self.chess_at(m.from);
        self.set_chess(m.to, chess);
        self.set_chess(m.from, Chess::None);
        self.zobrist_value = ZOBRIST_TABLE.apply_move(self.zobrist_value, m);
        self.zobrist_value_lock = ZOBRIST_TABLE_LOCK.apply_move(self.zobrist_value_lock, m);
        self.turn = m.player.next();
    }
    pub fn do_move(&mut self, m: &Move) {
        self.apply_move(m);
        self.distance += 1;
        self.move_history.push(m.clone());
    }
    pub fn undo_move(&mut self, m: &Move) {
        let chess = self.chess_at(m.to);
        self.set_chess(m.from, chess);
        self.set_chess(m.to, m.capture);
        self.zobrist_value = ZOBRIST_TABLE.undo_move(self.zobrist_value, m);
        self.zobrist_value_lock = ZOBRIST_TABLE_LOCK.undo_move(self.zobrist_value_lock, m);
        self.turn = m.player;
        self.distance -= 1;
        self.move_history.pop();
    }
    pub fn chess_at(&self, pos: Position) -> Chess {
        if in_board(pos) {
            self.chesses[pos.row as usize][pos.col as usize]
        } else {
            Chess::None
        }
    }
    pub fn set_chess(&mut self, pos: Position, chess: Chess) {
        self.chesses[pos.row as usize][pos.col as usize] = chess;
    }
    pub fn has_chess_between(&self, posa: Position, posb: Position) -> bool {
        if posa.row == posb.row {
            for j in posa.col.min(posb.col) + 1..posb.col.max(posa.col) {
                if self
                    .chess_at(Position::new(posa.row, j))
                    .chess_type()
                    .is_some()
                {
                    return true;
                }
            }
        } else if posa.col == posb.col {
            for i in posa.row.min(posb.row) + 1..posb.row.max(posa.row) {
                if self
                    .chess_at(Position::new(i, posa.col))
                    .chess_type()
                    .is_some()
                {
                    return true;
                }
            }
        }
        return false;
    }
    pub fn king_position(&self, player: Player) -> Option<Position> {
        if player == Player::Black {
            for i in 0..3 {
                for j in 3..6 {
                    if self.chess_at(Position::new(i, j)) == Chess::Black(ChessType::King) {
                        return Some(Position::new(i, j));
                    }
                }
            }
        } else {
            for i in 7..10 {
                for j in 3..6 {
                    if self.chess_at(Position::new(i, j)) == Chess::Red(ChessType::King) {
                        return Some(Position::new(i, j));
                    }
                }
            }
        }
        None
    }
    pub fn king_eye_to_eye(&self) -> bool {
        let posa = self.king_position(Player::Red).unwrap();
        let posb = self.king_position(Player::Black).unwrap();
        if posa.col == posb.col {
            !self.has_chess_between(posa, posb)
        } else {
            false
        }
    }
    pub fn is_checked(&self, player: Player) -> bool {
        let position_base = self.king_position(player).unwrap();

        // 是否被炮将军
        let targets = self.generate_move_for_chess_type(ChessType::Cannon, position_base);
        for pos in targets {
            if self.chess_at(pos).belong_to(player.next()) {
                if let Some(ChessType::Cannon) = self.chess_at(pos).chess_type() {
                    return true;
                }
            }
        }
        // 是否被车将军
        let targets = self.generate_move_for_chess_type(ChessType::Rook, position_base);
        for pos in targets {
            if self.chess_at(pos).belong_to(player.next()) {
                if let Some(ChessType::Rook) = self.chess_at(pos).chess_type() {
                    return true;
                }
            }
        }

        // 是否被马将军
        let mut targets = vec![];
        if self.chess_at(position_base.up(1).left(1)) == Chess::None {
            targets.push(position_base.up(2).left(1));
            targets.push(position_base.up(1).left(2));
        }
        if self.chess_at(position_base.down(1).left(1)) == Chess::None {
            targets.push(position_base.down(2).left(1));
            targets.push(position_base.down(1).left(2));
        }
        if self.chess_at(position_base.up(1).right(1)) == Chess::None {
            targets.push(position_base.up(2).right(1));
            targets.push(position_base.up(1).right(2));
        }
        if self.chess_at(position_base.down(1).right(1)) == Chess::None {
            targets.push(position_base.down(2).right(1));
            targets.push(position_base.down(1).right(2));
        }
        for pos in targets {
            if self.chess_at(pos).belong_to(player.next()) {
                if let Some(ChessType::Knight) = self.chess_at(pos).chess_type() {
                    return true;
                }
            }
        }

        // 是否被兵将军
        for pos in vec![
            position_base.left(1),
            position_base.right(1),
            if player == Player::Red {
                position_base.up(1)
            } else {
                position_base.down(1)
            },
        ] {
            if self.chess_at(pos).belong_to(player.next()) {
                if let Some(ChessType::Pawn) = self.chess_at(pos).chess_type() {
                    return true;
                }
            }
        }
        return self.king_eye_to_eye();
    }
    pub fn generate_move_for_chess_type(&self, ct: ChessType, position_base: Position) -> Vec<Position> {
        let mut targets = vec![];
        match ct {
            ChessType::King => {
                targets.append(&mut vec![
                    position_base.up(1),
                    position_base.down(1),
                    position_base.left(1),
                    position_base.right(1),
                ]);
            }
            ChessType::Advisor => {
                targets.append(&mut vec![
                    position_base.up(1).left(1),
                    position_base.up(1).right(1),
                    position_base.down(1).left(1),
                    position_base.down(1).right(1),
                ]);
            }
            ChessType::Bishop => {
                if self.chess_at(position_base.up(1).left(1)) == Chess::None {
                    targets.push(position_base.up(2).left(2));
                }
                if self.chess_at(position_base.up(1).right(1)) == Chess::None {
                    targets.push(position_base.up(2).right(2));
                }
                if self.chess_at(position_base.down(1).left(1)) == Chess::None {
                    targets.push(position_base.down(2).left(2));
                }
                if self.chess_at(position_base.down(1).right(1)) == Chess::None {
                    targets.push(position_base.down(2).right(2));
                }
            }
            ChessType::Knight => {
                if self.turn == Player::Red {
                    if self.chess_at(position_base.up(1)) == Chess::None {
                        targets.push(position_base.up(2).left(1));
                        targets.push(position_base.up(2).right(1));
                    }
                    if self.chess_at(position_base.down(1)) == Chess::None {
                        targets.push(position_base.down(2).left(1));
                        targets.push(position_base.down(2).right(1));
                    }
                } else {
                    if self.chess_at(position_base.down(1)) == Chess::None {
                        targets.push(position_base.down(2).left(1));
                        targets.push(position_base.down(2).right(1));
                    }
                    if self.chess_at(position_base.up(1)) == Chess::None {
                        targets.push(position_base.up(2).left(1));
                        targets.push(position_base.up(2).right(1));
                    }
                }

                if self.chess_at(position_base.left(1)) == Chess::None {
                    targets.push(position_base.up(1).left(2));
                    targets.push(position_base.down(1).left(2));
                }
                if self.chess_at(position_base.right(1)) == Chess::None {
                    targets.push(position_base.up(1).right(2));
                    targets.push(position_base.down(1).right(2));
                }
            }
            ChessType::Rook => {
                if self.turn == Player::Red {
                    for delta in 1..(position_base.row + 1) {
                        targets.push(position_base.up(delta));
                        if self.chess_at(position_base.up(delta)) != Chess::None {
                            break;
                        }
                    }
                    for delta in 1..(BOARD_HEIGHT - position_base.row) {
                        targets.push(position_base.down(delta));
                        if self.chess_at(position_base.down(delta)) != Chess::None {
                            break;
                        }
                    }
                } else {
                    for delta in 1..(position_base.row + 1) {
                        targets.push(position_base.up(delta));
                        if self.chess_at(position_base.up(delta)) != Chess::None {
                            break;
                        }
                    }
                    for delta in 1..(BOARD_HEIGHT - position_base.row) {
                        targets.push(position_base.down(delta));
                        if self.chess_at(position_base.down(delta)) != Chess::None {
                            break;
                        }
                    }
                }

                for delta in 1..(position_base.col + 1) {
                    targets.push(position_base.left(delta));
                    if self.chess_at(position_base.left(delta)) != Chess::None {
                        break;
                    }
                }
                for delta in 1..(BOARD_WIDTH - position_base.col) {
                    targets.push(position_base.right(delta));
                    if self.chess_at(position_base.right(delta)) != Chess::None {
                        break;
                    }
                }
            }
            ChessType::Cannon => {
                let mut has_chess = false;
                for delta in 1..(position_base.row + 1) {
                    if !has_chess {
                        if self.chess_at(position_base.up(delta)) != Chess::None {
                            has_chess = true;
                        } else {
                            targets.push(position_base.up(delta));
                        }
                    } else if self.chess_at(position_base.up(delta)) != Chess::None {
                        targets.push(position_base.up(delta));
                        break;
                    }
                }
                let mut has_chess = false;
                for delta in 1..(BOARD_HEIGHT - position_base.row) {
                    if !has_chess {
                        if self.chess_at(position_base.down(delta)) != Chess::None {
                            has_chess = true;
                        } else {
                            targets.push(position_base.down(delta));
                        }
                    } else if self.chess_at(position_base.down(delta)) != Chess::None {
                        targets.push(position_base.down(delta));
                        break;
                    }
                }
                let mut has_chess = false;
                for delta in 1..(position_base.col + 1) {
                    if !has_chess {
                        if self.chess_at(position_base.left(delta)) != Chess::None {
                            has_chess = true;
                        } else {
                            targets.push(position_base.left(delta));
                        }
                    } else if self.chess_at(position_base.left(delta)) != Chess::None {
                        targets.push(position_base.left(delta));
                        break;
                    }
                }
                let mut has_chess = false;
                for delta in 1..(BOARD_WIDTH - position_base.col) {
                    if !has_chess {
                        if self.chess_at(position_base.right(delta)) != Chess::None {
                            has_chess = true;
                        } else {
                            targets.push(position_base.right(delta));
                        }
                    } else if self.chess_at(position_base.right(delta)) != Chess::None {
                        targets.push(position_base.right(delta));
                        break;
                    }
                }
            }
            ChessType::Pawn => {
                // 过河兵可以左右走
                if !in_country(position_base.row, self.turn) {
                    targets.push(position_base.left(1));
                    targets.push(position_base.right(1));
                }
                if self.turn == Player::Black {
                    targets.push(position_base.down(1))
                } else {
                    targets.push(position_base.up(1));
                }
            }
        }
        targets
    }
    pub fn generate_move(&mut self, capture_only: bool) -> Vec<Move> {
        self.gen_counter += 1;
        let mut moves = vec![];
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                let position_base = Position::new(i, j);
                // 遍历每个行棋方的棋
                let chess = self.chess_at(position_base);
                if chess.belong_to(self.turn) {
                    if let Some(ct) = chess.chess_type() {
                        let targets = self.generate_move_for_chess_type(ct, position_base);
                        let move_base = Move {
                            player: self.turn,
                            from: position_base,
                            to: position_base,
                            chess,
                            capture: Chess::None,
                        };
                        for target in targets {
                            let valid = if ct == ChessType::King || ct == ChessType::Advisor {
                                // 帅和士要在九宫格内
                                in_palace(target, self.turn)
                            } else if ct == ChessType::Bishop {
                                // 象不能过河
                                in_country(target.row, self.turn) && in_board(target)
                            } else {
                                in_board(target)
                            };

                            if valid {
                                if !self.chess_at(target).belong_to(self.turn)
                                    && (!capture_only || self.chess_at(target).chess_type().is_some())
                                {
                                    moves.push(move_base.with_target(target, self.chess_at(target)));
                                }
                            }
                        }
                    }
                }
            }
        }
        moves.sort_by(|a, b| {
            (self.chess_at(b.to).value() - self.chess_at(b.from).value())
                .cmp(&(self.chess_at(a.to).value() - self.chess_at(a.from).value()))
        });
        moves
    }
    // 简单的评价，双方每个棋子的子力之和的差
    pub fn evaluate(&self, player: Player) -> i32 {
        let mut red_score = 0;
        let mut black_score = 0;
        for i in 0..BOARD_HEIGHT as usize {
            for j in 0..BOARD_WIDTH as usize {
                let chess = self.chess_at(Position::new(i as i32, j as i32));
                if let Some(ct) = chess.chess_type() {
                    let pos = if chess.belong_to(Player::Black) {
                        Position::new(i as i32, j as i32).flip()
                    } else {
                        Position::new(i as i32, j as i32)
                    };
                    let score = match ct {
                        ChessType::King => KING_VALUE_TABLE[pos.row as usize][pos.col as usize],
                        ChessType::Advisor => ADVISOR_VALUE_TABLE[pos.row as usize][pos.col as usize],
                        ChessType::Bishop => BISHOP_VALUE_TABLE[pos.row as usize][pos.col as usize],
                        ChessType::Knight => KNIGHT_VALUE_TABLE[pos.row as usize][pos.col as usize],
                        ChessType::Rook => ROOK_VALUE_TABLE[pos.row as usize][pos.col as usize],
                        ChessType::Cannon => CANNON_VALUE_TABLE[pos.row as usize][pos.col as usize],
                        ChessType::Pawn => PAWN_VALUE_TABLE[pos.row as usize][pos.col as usize],
                    };
                    if chess.belong_to(Player::Black) {
                        black_score += score
                    } else {
                        red_score += score
                    }
                }
            }
        }
        if player == Player::Red {
            red_score - black_score + INITIATIVE_BONUS
        } else {
            black_score - red_score + INITIATIVE_BONUS
        }
    }
    pub fn find_record(&self) -> Option<Record> {
        if let Some(record) = &self.records[(self.zobrist_value & (RECORD_SIZE - 1) as u64) as usize] {
            if record.zobrist_lock == self.zobrist_value_lock && self.turn == record.turn {
                Some(record.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn add_record(&mut self, record: Record) {
        if let Some(old_record) = &self.records[(self.zobrist_value & (RECORD_SIZE - 1) as u64) as usize] {
            // 如果已存在，用深度较大的覆盖，depth越小，深度越大
            if record.depth < old_record.depth {
                self.records[(self.zobrist_value & (RECORD_SIZE - 1) as u64) as usize] = Some(record);
            }
        } else {
            self.records[(self.zobrist_value & (RECORD_SIZE - 1) as u64) as usize] = Some(record);
        }
    }
    pub fn alpha_beta_pvs(&mut self, depth: i32, mut alpha: i32, beta: i32) -> (i32, Option<Move>) {
        // if let Some(record) = self.find_record() {
        //     if record.depth <= depth {
        //         return (record.value, record.best_move);
        //     }
        // }
        if depth == 0 {
            self.counter += 1;
            return (self.quies(alpha, beta), None);
        }
        let mut count = 0; // 记录尝试了多少种着法

        // 优先尝试迭代深度搜索的上一层搜索结果
        let mut moves = self.generate_move(false);
        // 如果符合上次搜索的着法线路，那么优先按此线路搜索下去
        for (i, m) in self.best_moves_last.iter().enumerate() {
            if let Some(ml) = self.move_history.get(i) {
                if m != ml {
                    break;
                }
            } else {
                moves.insert(0, m.clone());
                break;
            }
        }
        let mut best_move = None;
        for m in moves {
            self.do_move(&m);
            if self.is_checked(self.turn.next()) {
                self.undo_move(&m);
                continue;
            }
            count = count + 1;
            // 先使用0宽窗口进行搜索
            let (v, bmt) = self.alpha_beta_pvs(depth - 1, -(alpha + 1), -alpha);

            let mut best_value = -v;
            let mut bm = bmt;
            if best_value == MIN || (best_value > alpha && best_value < beta) {
                let (v, bmt) = self.alpha_beta_pvs(depth - 1, -beta, -alpha);
                // self.add_record(Record {
                //     value: -v,
                //     depth,
                //     best_move: bmt.clone(),
                //     zobrist_lock: self.zobrist_value_lock,
                //     turn: self.turn,
                // });
                best_value = -v;
                bm = bmt;
            }

            // let (v, bmt) = self.alpha_beta(depth - 1, -beta, -alpha);
            // let mut best_value = -v;
            // let mut bm = bmt;

            if best_value >= beta {
                self.undo_move(&m);
                return (best_value, None);
            }
            if best_value > alpha {
                alpha = best_value;
                best_move = Some(m.clone());
            }

            self.undo_move(&m);
        }

        // 如果尝试的着法数为0,说明已经被绝杀
        // 深度减分，深度越小，说明越早被将死，局面分应该越低，由于depth是递减的，
        // 所以深度越小，depth越大，减去depth的局面分就越低
        return (if count == 0 { KILL - depth } else { alpha }, best_move);
    }
    pub fn quies(&mut self, mut alpha: i32, beta: i32) -> i32 {
        if self.distance > MAX_DEPTH {
            return self.evaluate(self.turn);
        }
        let v = self.evaluate(self.turn);
        if v >= beta {
            return beta;
        }
        if v > alpha {
            alpha = v
        }
        let moves = if self.is_checked(self.turn.next()) {
            self.generate_move(false)
        } else {
            self.generate_move(true)
        };
        for m in moves {
            self.do_move(&m);
            if self.is_checked(self.turn.next()) {
                self.undo_move(&m);
                continue;
            }
            let v = -self.quies(-beta, -alpha);
            self.undo_move(&m);
            if v >= beta {
                return beta;
            }
            if v > alpha {
                alpha = v;
            }
        }
        return alpha;
    }
    pub fn iterative_deepening(&mut self, max_depth: i32) -> (i32, Option<Move>) {
        if max_depth > 3 {
            for depth in 3..max_depth + 1 {
                // self.records = vec![RECORD_NONE; RECORD_SIZE as usize];
                let (v, bm) = self.alpha_beta_pvs(depth, MIN, MAX);
                if depth == max_depth {
                    println!("第{}层: {:?}", depth, bm);
                    return (v, bm);
                }
                self.best_moves_last = vec![];
                self.best_moves_last.reverse();
                println!("第{}层: {:?}", depth, self.best_moves_last);
            }
        } else {
            // self.records = vec![RECORD_NONE; RECORD_SIZE as usize];
            return self.alpha_beta_pvs(max_depth, MIN, MAX);
        }
        (0, None)
    }
}

#[cfg(test)]
mod tests {
    use crate::board::*;

    #[test]
    fn test_generate_move() {
        let mut board = Board::init();
        for i in 0..1_000 {
            board.generate_move(false);
        }
        assert_eq!(Board::init().generate_move(false).len(), 5 + 24 + 4 + 4 + 4 + 2 + 1);
    }
    #[test]
    fn test_is_checked() {
        let mut board = Board::init();
        for _i in 0..10_000 {
            board.is_checked(Player::Red);
        }
        assert_eq!(Board::init().generate_move(false).len(), 5 + 24 + 4 + 4 + 4 + 2 + 1);
    }
    #[test]
    fn test_move_and_unmove() {
        let mut board = Board::init();
        for _i in 0..8_000 {
            let m = Move {
                player: Player::Red,
                from: Position::new(0, 0),
                to: Position::new(1, 0),
                chess: Chess::Red(ChessType::Rook),
                capture: Chess::None,
            };
            board.apply_move(&m);
            board.undo_move(&m);
        }
        assert_eq!(Board::init().generate_move(false).len(), 5 + 24 + 4 + 4 + 4 + 2 + 1);
    }

    #[test]
    fn test_evaluate() {
        let mut board = Board::init();
        board.apply_move(&Move {
            player: Player::Red,
            from: Position { row: 9, col: 8 },
            to: Position { row: 7, col: 8 },
            chess: Chess::Red(ChessType::Rook),
            capture: Chess::None,
        });
        for i in 0..10_000 {
            board.evaluate(Player::Red);
        }
        assert_eq!(board.evaluate(Player::Red), 7);
    }

    #[test]
    fn test_alpha_beta_pvs() {
        println!("{:?}", Board::init().alpha_beta_pvs(1, MIN, MAX));
        // println!("{:?}", Board::init().alpha_beta_pvs(2, MIN, MAX));
        // println!("{:?}", Board::init().alpha_beta_pvs(3, MIN, MAX));
        // println!("{:?}", Board::init().alpha_beta_pvs(4, MIN, MAX));
        // let mut board = Board::init();
        // let rst = board.minimax(5, Player::Red, i32::MIN, i32::MAX);
        // let counter = board.counter;
        // println!("{} \n {:?}", counter, rst); // 跳马
        //                                       /* */
        // println!("{:?}", Board::init().alpha_beta_pvs(6, MIN, MAX)); // 跳马
    }

    #[test]
    fn test_from_fen() {
        let fen = "rnb1kabnr/4a4/1c5c1/p1p3p2/4N4/8p/P1P3P1P/2C4C1/9/RNBAKAB1R w - - 0 1 moves e5d7";
        println!("{:?}", Board::from_fen(fen).chesses);
    }

    #[test]
    fn test_king_eye_to_eye() {
        let board = Board::from_fen("rnbakabnr/9/1c5c1/9/9/9/9/1C5C1/9/RNBAKABNR w - - 0 1");
        println!("{:?}", board.chesses);
        println!("{}", board.king_eye_to_eye());
        let board = Board::init();
        println!("{}", board.king_eye_to_eye());
    }
}
