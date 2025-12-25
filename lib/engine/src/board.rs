/*
 * 详细中文注释 - 象棋棋盘模块（Board 与棋子表示）
 *
 * 设计要点
 * - 棋盘尺寸为 9 列 x 10 行，红方在下，黑方在上
 * - 棋子用 Chess 枚举表示，分黑方/红方与具体棋种；Chess::None 表示空格
 * - ChessType 定义了具体棋子的类型（King/Advisor/Bishop/Knight/Rook/Cannon/Pawn）及其属性
 * - Player 表示当前轮到的玩家（Red/Black）
 * - Position 表示棋子在棋盘上的坐标，行列从 0 开始，内部实现与坐标系紧密绑定
 * - Move 记录一次落子信息，包括起点、终点、走子的棋种以及吃子信息
 * - Board 保存当前局面的完整状态：棋子排布、轮到哪一方、历史记录、Zobrist 哈希量等
 *
 * 主要功能（保持与现有实现一致）
 * - 初始化棋盘、从 FEN/类 FEN 字符串加载局面
 * - 移动的应用、撤销、以及是否为合法走法的判定
 * - 走法生成、对局面中的吃子逻辑以及是否将军/吃子检查
 * - 简单评估函数、以及简化的搜索（α-β、PV 倍增、迭代深化、静态棋力评估）
 * - Zobrist 哈希值的维护与置换表接口初步实现（与外部 zobrist.rs 配合）
 *
 * 注意
 * - 本注释仅为帮助理解代码设计与实现细节，具体行为以代码为准
 * - 任何对实现的改动都应保持现有接口不变，确保编译通过
 */

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
        if self == &Player::Red { 0 } else { 1 }
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

#[derive(Clone, Debug)]
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
    // 杀手走法表：每层深度保存2个最佳走法
    pub killer_table: Vec<[Option<Move>; 2]>,
    // 历史启发表：记录每个走法的历史得分
    // 索引：from_square_index * 90 + to_square_index
    // 总共 90 * 90 = 8100 种可能的走法
    pub history_table: Vec<i32>,
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
    // 初始化标准象棋开局局面
    // 返回一个新的 Board 实例，棋子按标准布局摆放，红方先手
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
            records: vec![None; RECORD_SIZE as usize],
            zobrist_value: 0,
            zobrist_value_lock: 0,
            distance: 0,
            select_pos: Position { row: 1, col: 1 },
            // 初始化杀手走法表：每层深度2个空走法
            killer_table: vec![[None, None]; MAX_DEPTH as usize],
            // 初始化历史启发表：所有位置初始化为0
            // 90个格子 * 90个格子 = 8100 种可能的走法
            history_table: vec![0; 90 * 90],
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
            records: vec![None; RECORD_SIZE as usize],
            zobrist_value: 0,
            zobrist_value_lock: 0,
            distance: 0,
            select_pos: Position { row: 1, col: 1 },
            killer_table: vec![[None, None]; MAX_DEPTH as usize],
            history_table: vec![0; 90 * 90],
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
    // 应用走子到棋盘，但不更新历史记录（用于临时模拟）
    // 参数 m: 要应用的走子
    pub fn apply_move(&mut self, m: &Move) {
        let chess = self.chess_at(m.from);
        self.set_chess(m.to, chess);
        self.set_chess(m.from, Chess::None);
        self.zobrist_value = ZOBRIST_TABLE.apply_move(self.zobrist_value, m);
        self.zobrist_value_lock = ZOBRIST_TABLE_LOCK.apply_move(self.zobrist_value_lock, m);
        self.turn = m.player.next();
    }
    // 执行走子并更新历史记录（用于实际游戏）
    // 参数 m: 要执行的走子
    pub fn do_move(&mut self, m: &Move) {
        self.apply_move(m);
        self.distance += 1;
        self.move_history.push(m.clone());
    }
    // 撤销走子并恢复历史记录（用于回溯）
    // 参数 m: 要撤销的走子
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

    // 执行空着 (Null Move)：只交换走棋方
    pub fn do_null_move(&mut self) {
        self.turn = self.turn.next();
        self.distance += 1;
    }

    // 撤销空着
    pub fn undo_null_move(&mut self) {
        self.turn = self.turn.next();
        self.distance -= 1;
    }

    // 判断当前局面是否适合使用 null move
    // 当己方子力足够时才使用 (避免残局中误判)
    fn null_move_okay(&self) -> bool {
        // 简单检查：至少有一个车或炮
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                let chess = self.chesses[row as usize][col as usize];
                if let Some(player) = chess.player() {
                    if player == self.turn {
                        if let Some(ct) = chess.chess_type() {
                            if ct == ChessType::Rook || ct == ChessType::Cannon {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
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

    // 检查走子是否合法（包括规则和将军检查）
    // 参数 m: 要检查的走子
    // 返回: true 如果合法，否则 false
    pub fn is_move_legal(&self, m: &Move) -> bool {
        let chess = self.chess_at(m.from);

        // 1. 检查当前走棋方是否拥有该棋
        if !chess.belong_to(self.turn) {
            return false;
        }

        // 2. 目标格子若有同色棋子则不可走
        if self.chess_at(m.to).belong_to(self.turn) {
            return false;
        }

        // 3. 根据棋种判定走法是否合法
        if let Some(ct) = chess.chess_type() {
            if !self.is_move_valid_for_chess_type(ct, m.from, m.to) {
                return false;
            }
        } else {
            // 起手位置无棋子
            return false;
        }

        // 4. 走子后是否将军，若将军则不合法
        let mut temp_board = self.clone();
        // capture 字段在外部来源时可能不可靠，这里构造一个完整走法
        let mut complete_move = m.clone();
        complete_move.capture = temp_board.chess_at(m.to);

        temp_board.apply_move(&complete_move);
        if temp_board.is_checked(self.turn) {
            return false;
        }

        true
    }

    fn is_move_valid_for_chess_type(&self, ct: ChessType, from: Position, to: Position) -> bool {
        if !in_board(to) {
            return false;
        }
        match ct {
            ChessType::King => (from.row - to.row).abs() + (from.col - to.col).abs() == 1 && in_palace(to, self.turn),
            ChessType::Advisor => {
                (from.row - to.row).abs() == 1 && (from.col - to.col).abs() == 1 && in_palace(to, self.turn)
            }
            ChessType::Bishop => {
                (from.row - to.row).abs() == 2
                    && (from.col - to.col).abs() == 2
                    && in_country(to.row, self.turn)
                    && self.chess_at(Position::new((from.row + to.row) / 2, (from.col + to.col) / 2)) == Chess::None
            }
            ChessType::Knight => {
                let row_diff = (from.row - to.row).abs();
                let col_diff = (from.col - to.col).abs();
                if !((row_diff == 1 && col_diff == 2) || (row_diff == 2 && col_diff == 1)) {
                    return false;
                }

                if row_diff == 2 {
                    // 跳马：中间是否有阻挡
                    if self.chess_at(Position::new((from.row + to.row) / 2, from.col)) != Chess::None {
                        return false;
                    }
                } else {
                    // 跳马：横向阻挡
                    if self.chess_at(Position::new(from.row, (from.col + to.col) / 2)) != Chess::None {
                        return false;
                    }
                }
                true
            }
            ChessType::Rook => (from.row == to.row || from.col == to.col) && !self.has_chess_between(from, to),
            ChessType::Cannon => {
                if from.row == to.row || from.col == to.col {
                    if self.chess_at(to) == Chess::None {
                        !self.has_chess_between(from, to)
                    } else {
                        self.count_chess_between(from, to) == 1
                    }
                } else {
                    false
                }
            }
            ChessType::Pawn => {
                // 兵/卒的推进规则
                let forward_ok = if self.turn == Player::Red {
                    to.row == from.row - 1 && to.col == from.col
                } else {
                    to.row == from.row + 1 && to.col == from.col
                };
                if in_country(from.row, self.turn) {
                    forward_ok
                } else {
                    let side_ok = from.row == to.row && (from.col - to.col).abs() == 1;
                    forward_ok || side_ok
                }
            }
        }
    }

    pub fn count_chess_between(&self, posa: Position, posb: Position) -> i32 {
        let mut count = 0;
        if posa.row == posb.row {
            for j in posa.col.min(posb.col) + 1..posb.col.max(posa.col) {
                if self
                    .chess_at(Position::new(posa.row, j))
                    .chess_type()
                    .is_some()
                {
                    count += 1;
                }
            }
        } else if posa.col == posb.col {
            for i in posa.row.min(posb.row) + 1..posb.row.max(posa.row) {
                if self
                    .chess_at(Position::new(i, posa.col))
                    .chess_type()
                    .is_some()
                {
                    count += 1;
                }
            }
        }
        count
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
        for pos in [
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
    // 生成当前玩家的所有合法走子
    // 参数 capture_only: true 只生成吃子走子，false 生成所有走子
    // 返回: 合法走子的向量，按优先级排序
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
    // 简单的评价函数，计算双方棋子的子力差（包括位置加成）
    // 参数 player: 当前评估的玩家
    // 返回: 评估分数，正数表示 player 优势
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

    // 计算走法的历史启发索引
    // 基于起点和终点位置
    // 棋盘是 10x9 的，所以总共 90 个格子
    // 每个起点-终点对应一个索引：from_index * 90 + to_index
    fn history_index(&self, mv: &Move) -> usize {
        let from_idx = (mv.from.row * 9 + mv.from.col) as usize;
        let to_idx = (mv.to.row * 9 + mv.to.col) as usize;
        from_idx * 90 + to_idx
    }

    // 更新杀手走法表
    // 当找到一个好的走法时调用
    fn update_killer_move(&mut self, mv: &Move, depth: usize) {
        if depth >= self.killer_table.len() {
            return;
        }
        // 如果不是第一个杀手走法，则更新
        if let Some(killer1) = &self.killer_table[depth][0] {
            if killer1 != mv {
                self.killer_table[depth][1] = self.killer_table[depth][0].clone();
                self.killer_table[depth][0] = Some(mv.clone());
            }
        } else {
            self.killer_table[depth][0] = Some(mv.clone());
        }
    }

    // 更新历史启发表
    // depth^2 作为奖励，深度越大越重要
    fn update_history(&mut self, mv: &Move, depth: i32) {
        let idx = self.history_index(mv);
        if idx < self.history_table.len() {
            self.history_table[idx] += depth * depth;
        }
    }

    // 获取走法的历史得分
    fn get_history_score(&self, mv: &Move) -> i32 {
        let idx = self.history_index(mv);
        if idx < self.history_table.len() {
            self.history_table[idx]
        } else {
            0
        }
    }

    // 改进的走法排序
    // 排序优先级：Hash Move > Killer Move > MVV/LVA > History Heuristic
    fn sort_moves(&self, moves: &mut Vec<Move>, hash_move: Option<&Move>) {
        let depth = self.distance as usize;
        let killer1 = if depth < self.killer_table.len() {
            self.killer_table[depth][0].as_ref()
        } else {
            None
        };
        let killer2 = if depth < self.killer_table.len() {
            self.killer_table[depth][1].as_ref()
        } else {
            None
        };

        // 为每个走法计算排序分数
        let mut move_scores: Vec<(Move, i32)> = moves
            .iter()
            .map(|mv| {
                let mut score = 0;

                // 最高优先级：Hash Move
                if let Some(hm) = hash_move {
                    if mv == hm {
                        return (mv.clone(), i32::MAX);
                    }
                }

                // 杀手走法
                if let Some(k1) = killer1 {
                    if mv == k1 {
                        return (mv.clone(), i32::MAX - 1);
                    }
                }
                if let Some(k2) = killer2 {
                    if mv == k2 {
                        return (mv.clone(), i32::MAX - 2);
                    }
                }

                // MVV/LVA (Most Valuable Victim / Least Valuable Aggressor)
                // 吃子走法优先，吃价值高的子且用价值低的子吃
                if mv.capture != Chess::None {
                    score += mv.capture.value() * 10 - mv.chess.value();
                }

                // 历史启发分数
                score += self.get_history_score(mv);

                (mv.clone(), score)
            })
            .collect();

        // 按分数降序排序
        move_scores.sort_by(|a, b| b.1.cmp(&a.1));

        // 更新原 moves 向量
        *moves = move_scores.into_iter().map(|(mv, _)| mv).collect();
    }
    // Alpha-Beta 搜索与 PV 倍增（主搜索函数）
    // 参数 depth: 搜索深度
    // 参数 alpha: Alpha 值（下界）
    // 参数 beta: Beta 值（上界）
    // 参数 allow_null: 是否允许 null move pruning
    // 返回: (评估分数, 最佳走子)
    fn alpha_beta_pvs_internal(&mut self, depth: i32, mut alpha: i32, beta: i32, allow_null: bool) -> (i32, Option<Move>) {
        // 尝试从置换表获取结果
        let hash_move = if let Some(record) = self.find_record() {
            if record.depth <= depth {
                return (record.value, record.best_move);
            }
            record.best_move
        } else {
            None
        };

        if depth == 0 {
            self.counter += 1;
            return (self.quies(alpha, beta), None);
        }

        // Null Move Pruning
        // 当不在被将军状态，且允许 null move，且局面适合时尝试
        const NULL_MOVE_REDUCTION: i32 = 2;
        if allow_null && depth >= 3 && !self.is_checked(self.turn) && self.null_move_okay() {
            self.do_null_move();
            let (v, _) = self.alpha_beta_pvs_internal(depth - NULL_MOVE_REDUCTION - 1, -beta, -beta + 1, false);
            self.undo_null_move();
            if -v >= beta {
                return (beta, None); // Fail-high cutoff
            }
        }

        let mut count = 0; // 记录尝试了多少种着法

        // 生成所有走法
        let mut moves = self.generate_move(false);

        // 使用改进的走法排序
        self.sort_moves(&mut moves, hash_move.as_ref());

        let mut best_move = None;
        let mut best_value = MIN;

        for (i, m) in moves.iter().enumerate() {
            self.do_move(&m);
            if self.is_checked(self.turn.next()) {
                self.undo_move(&m);
                continue;
            }
            count += 1;

            let v = if i == 0 {
                // 第一个走法用全窗口搜索
                -self.alpha_beta_pvs_internal(depth - 1, -beta, -alpha, true).0
            } else {
                // 后续走法先用 null-window 搜索
                let scout = -self.alpha_beta_pvs_internal(depth - 1, -(alpha + 1), -alpha, false).0;
                if scout > alpha && scout < beta {
                    // 如果在窗口内，重新用全窗口搜索
                    -self.alpha_beta_pvs_internal(depth - 1, -beta, -alpha, true).0
                } else {
                    scout
                }
            };

            self.undo_move(&m);

            if v > best_value {
                best_value = v;
                if v >= beta {
                    // Beta 截断：更新 killer moves 和 history
                    self.update_killer_move(&m, self.distance as usize);
                    self.update_history(&m, depth);
                    // 保存到置换表
                    self.add_record(Record {
                        value: v,
                        depth,
                        best_move: Some(m.clone()),
                        zobrist_lock: self.zobrist_value_lock,
                        turn: self.turn,
                    });
                    return (v, Some(m.clone()));
                }
                if v > alpha {
                    alpha = v;
                    best_move = Some(m.clone());
                }
            }
        }

        // 如果尝试的着法数为0,说明已经被绝杀
        if count == 0 {
            return (KILL - depth, None);
        }

        // 保存到置换表
        if best_move.is_some() {
            self.add_record(Record {
                value: best_value,
                depth,
                best_move: best_move.clone(),
                zobrist_lock: self.zobrist_value_lock,
                turn: self.turn,
            });
        }

        (best_value, best_move)
    }

    // 公共接口：Alpha-Beta 搜索入口
    pub fn alpha_beta_pvs(&mut self, depth: i32, alpha: i32, beta: i32) -> (i32, Option<Move>) {
        self.alpha_beta_pvs_internal(depth, alpha, beta, true)
    }

    // 静态搜索（Quiescence Search），处理吃子序列
    // 参数 alpha: Alpha 值
    // 参数 beta: Beta 值
    // 返回: 静态评估分数
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
    // 迭代深化搜索（Iterative Deepening），逐步增加深度
    // 参数 max_depth: 最大搜索深度
    // 返回: (最终评估分数, 最佳走子)
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
