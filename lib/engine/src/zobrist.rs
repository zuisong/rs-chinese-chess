/*
 详细中文注释 - Zobrist 哈希实现（棋盘状态哈希）

 设计要点
 - Zobrist 哈希用于对棋盘状态进行快速、可更新的唯一标识，便于实现置换表（TT）等优化
 - hash_table[layer][position_index][piece_type]，其中 layer 表示棋子颜色（0/1），position_index 为 i*width + j
 - 通过对棋子位置与类型进行异或操作实现哈希的增量更新

 关键约定
 - calc_chesses 用于计算当前棋盘的哈希值
 - apply_move/undo_move 用于在落子前后更新哈希值，确保棋局状态一致性
*/
use crate::board::{BOARD_HEIGHT, BOARD_WIDTH, Chess, Move};

#[derive(Debug)]
pub struct Zobristable {
    hash_table: [[[u64; 7]; 90]; 2],
}

fn rand64() -> u64 {
    fastrand::u64(u64::MIN..=u64::MAX)
}

impl Zobristable {
    pub fn new() -> Self {
        let mut z = Zobristable {
            hash_table: [[[0u64; 7]; 90]; 2],
        };
        for l in 0..2 {
            for m in 0..90 {
                for n in 0..7 {
                    z.hash_table[l][m][n] = rand64();
                }
            }
        }
        z
    }
    pub fn calc_chesses(&self, chesses: &[[Chess; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize]) -> u64 {
        let mut value = 0 as u64;
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                let chess = chesses[i as usize][j as usize];
                if let Some(ct) = chess.chess_type() {
                    value ^= self.hash_table[chess.player().unwrap().value() as usize][(i * BOARD_WIDTH + j) as usize]
                        [ct.value() as usize];
                }
            }
        }
        value
    }
    pub fn apply_move(&self, origin: u64, m: &Move) -> u64 {
        let mut value = origin;
        // 把棋子从原位置拿起来
        value ^= self.hash_table[m.chess.player().unwrap().value() as usize]
            [(m.from.row * BOARD_WIDTH + m.from.col) as usize][m.chess.chess_type().unwrap().value() as usize];
        // 放到新的位置
        value ^= self.hash_table[m.chess.player().unwrap().value() as usize]
            [(m.to.row * BOARD_WIDTH + m.to.col) as usize][m.chess.chess_type().unwrap().value() as usize];
        // 如果有吃子，把被吃掉的子拿起来
        if let Some(ct) = m.capture.chess_type() {
            value ^= self.hash_table[m.capture.player().unwrap().value() as usize]
                [(m.to.row * BOARD_WIDTH + m.to.col) as usize][ct.value() as usize];
        }
        value
    }
    pub fn undo_move(&self, origin: u64, m: &Move) -> u64 {
        // 由于zobrist是异或运算，所以
        // undo_move与apply_move是一样的
        self.apply_move(origin, m)
    }
}

#[cfg(test)]
mod test {
    use crate::board::*;
    use crate::zobrist::*;

    #[test]
    fn test_zobrist() {
        println!("{}", Zobristable::new().calc_chesses(&Board::init().chesses));
    }

    #[test]
    fn test_zobrist_move() {
        let zorbis_table = Zobristable::new();
        let hash = zorbis_table.calc_chesses(&Board::init().chesses);
        let m = Move {
            player: crate::board::Player::Red,
            from: Position::new(0, 0),
            to: Position::new(2, 0),
            chess: Chess::Black(ChessType::Rook),
            capture: Chess::None,
        };
        assert_ne!(hash, zorbis_table.apply_move(hash, &m));
        let hash_after = zorbis_table.undo_move(zorbis_table.apply_move(hash, &m), &m);
        assert_eq!(hash, hash_after);
    }
}
