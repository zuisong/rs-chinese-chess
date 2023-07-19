use crate::game::Turn::{Black, Red};
use ChessType::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum ChessType {
    车, //  Car,
    马, //  Horse,
    象, //  Elephant,
    士, //  Advisor,
    帅, //  King,
    炮, //  Cannon,
    兵, //  Soldier,
}

#[derive(Debug)]
pub struct Chess {
    chess_type: ChessType,
    pub turn: Turn,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Turn {
    Red,
    Black,
}

impl Chess {
    fn can_move_to(&self, pos: &Position, game: &ChineseChess) -> bool {
        if let Some(chess) = game.get_chess(pos) {
            if chess.turn == self.turn {
                // 目标位置有棋子,且颜色相同,不能吃
                return false;
            }
        }

        let Position { x: x1, y: y1 } = self.position;
        let Position { x: x2, y: y2 } = *pos;

        if x2 > 8 || x2 < 0 || y2 < 0 || y2 > 9 {
            // 走出了棋盘区域
            return false;
        }

        match self.chess_type {
            车 => {
                // 车:直线移动,不能越过其他棋子

                // 同一行或同一列
                if x1 != x2 && y1 != y2 {
                    return false;
                }

                let mut x = x1;
                let mut y = y1;
                loop {
                    if x < x2 {
                        x += 1;
                    } else if x > x2 {
                        x -= 1;
                    }
                    if y < y2 {
                        y += 1;
                    } else if y > y2 {
                        y -= 1;
                    }
                    if x == x2 && y == y2 {
                        return true;
                    }
                    // 检查路径上是否有其他棋子
                    if game.has_chess(&Position { x, y }) {
                        return false;
                    }
                }
            }
            马 => {
                // 马:日字走法,可以越过其他棋子
                if (x1 - x2).abs() * (y1 - y2).abs() != 2 {
                    return false;
                }
                // 别马脚判定规则
                match (x2 - x1, y2 - y1) {
                    (2, _) => !game.has_chess(&Position { x: x1 + 1, y: y1 }),
                    (-2, _) => !game.has_chess(&Position { x: x1 - 1, y: y1 }),
                    (_, 2) => !game.has_chess(&Position { x: x1, y: y1 + 1 }),
                    (_, -2) => !game.has_chess(&Position { x: x1, y: y1 - 1 }),
                    (_, _) => unreachable!(),
                }
            }
            象 => {
                // 象:斜线移动,不能越过其他棋子
                (x1 - x2).abs() == 2
                    && (y1 - y2).abs() == 2
                    // 别象腿的情况是不能走的
                    && !game.has_chess(&Position {
                        x: (x1 + x2) / 2,
                        y: (y1 + y2) / 2,
                    })
            }
            士 => {
                // 士:斜线移动
                (x1 - x2).abs() * (y1 - y2).abs() == 1 && Chess::in_nine_palace(x2, y2)
            }
            帅 => {
                // 帅:一步一格
                (x1 - x2).abs() + (y1 - y2).abs() == 1 && Chess::in_nine_palace(x2, y2)
            }
            炮 => {
                // 炮:直线移动,可以越过其他棋子,但必须是吃子

                // 同一行或同一列
                if x1 != x2 && y1 != y2 {
                    return false;
                }

                let mut x = self.position.x;
                let mut y = self.position.y;
                let mut skipped = false;
                loop {
                    if x < x2 {
                        x += 1;
                    } else if x > x2 {
                        x -= 1;
                    }
                    if y < y2 {
                        y += 1;
                    } else if y > y2 {
                        y -= 1;
                    }
                    if x == x2 && y == y2 {
                        if skipped {
                            // 跳过棋子了 只能吃
                            return game.has_chess(pos);
                        } else {
                            // 没有跳过棋子 不能吃
                            return !game.has_chess(pos);
                        }
                    }
                    // 检查路径上是否有其他棋子
                    if game.has_chess(&Position { x, y }) {
                        if skipped {
                            // 离目标有多个棋子 不可以走
                            return false;
                        } else {
                            skipped = true;
                        }
                    }
                }
            }
            兵 => {
                // 己方将的的y坐标
                let king_y = game
                    .chessmen
                    .iter()
                    .find(|it| it.chess_type == ChessType::帅 && it.turn == self.turn)
                    .map(|it| it.position.y)
                    .unwrap();
                let over_river = if king_y <= 4 { y1 >= 5 } else { y1 <= 4 };

                // 兵:直线移动,不能越过其他棋子
                // 没过河, 只能前进
                // 过了河, 可以左右和前进
                (x1 == x2 && ((king_y <= 4 && y2 == y1 + 1) || (king_y >= 5 && y2 == y1 - 1)))
                    || (over_river && y1 == y2 && (x2 - x1).abs() == 1)
            }
        }
    }
    fn in_nine_palace(x: i32, y: i32) -> bool {
        // 两个九宫格区域
        (3..=5).contains(&x) && ((0..=2).contains(&y) || (7..=9).contains(&y))
    }
    pub fn name_str(&self) -> &'static str {
        match self.chess_type {
            车 => "车",
            马 => "马",
            象 => "象",
            士 => "士",
            帅 => "帅",
            炮 => "炮",
            兵 => "兵",
        }
    }
}

impl From<(ChessType, Turn, (i32, i32))> for Chess {
    fn from(value: (ChessType, Turn, (i32, i32))) -> Chess {
        let (chess_type, turn, (x, y)) = value;
        Chess {
            chess_type,
            turn,
            position: Position { x, y },
        }
    }
}

pub struct ChineseChess {
    pub chessmen: Vec<Chess>,                 // 棋盘上的棋子
    selected: Option<usize>,                  // 当前选中的棋子序号
    cur_turn: Turn,                           // 当前走棋方
    history: Vec<(Turn, Position, Position)>, // 历史记录 方便撤回
}
impl ChineseChess {
    fn has_chess(&self, pos: &Position) -> bool {
        self.chessmen
            .iter()
            .any(|c| c.position == *pos)
    }
    fn get_chess(&self, pos: &Position) -> Option<&Chess> {
        self.chessmen
            .iter()
            .find(|c| c.position == *pos)
    }
    pub fn click(&mut self, pos: &Position) {
        let selected = self.select(pos);
        if !selected {
            self.move_to(pos);
        }
    }
    fn select(&mut self, pos: &Position) -> bool {
        for (i, chess) in self
            .chessmen
            .iter()
            .enumerate()
        {
            if chess.position == *pos && chess.turn == self.cur_turn {
                self.selected = Some(i);
                return true;
            }
        }
        return false;
    }
    fn move_to(&mut self, pos: &Position) {
        // eat chess
        let mut eat_chess = None;
        for (i, chess) in self
            .chessmen
            .iter()
            .enumerate()
        {
            if chess.position == *pos && chess.turn != self.cur_turn {
                eat_chess = Some(i);
            }
        }
        // move to target
        if let Some(selected) = self.selected {
            let chess = &self.chessmen[selected];
            if chess.can_move_to(&pos, &self) {
                self.history
                    .push((self.cur_turn, chess.position, pos.clone()));
                let chess = &mut self.chessmen[selected];
                chess.position = pos.clone();
                self.cur_turn = match self.cur_turn {
                    Red => Black,
                    Black => Red,
                }; // 改变走棋方
                self.selected = None;
                if let Some(idx) = eat_chess {
                    self.chessmen
                        .remove(idx);
                }
                return;
            }
        }
    }
    #[allow(dead_code)]
    fn replay_history(&mut self) {
        let old = std::mem::replace(self, ChineseChess::default());
        for (_a, _b, _c) in old.history {}
    }
}
impl Default for ChineseChess {
    fn default() -> ChineseChess {
        let chessmen: Vec<Chess> = vec![
            // 半边棋子初始位置
            (车, (0, 0)),
            (车, (8, 0)),
            (马, (7, 0)),
            (马, (1, 0)),
            (象, (6, 0)),
            (象, (2, 0)),
            (士, (5, 0)),
            (士, (3, 0)),
            (帅, (4, 0)),
            (炮, (1, 2)),
            (炮, (7, 2)),
            (兵, (6, 3)),
            (兵, (4, 3)),
            (兵, (2, 3)),
            (兵, (0, 3)),
            (兵, (8, 3)),
        ]
        .into_iter()
        .flat_map(|(c, (x, y))| [(c, Black, (x, y)), (c, Red, (8 - x, 9 - y))])
        .map(Into::into)
        .collect();
        return ChineseChess {
            chessmen,
            cur_turn: Turn::Red,
            history: Default::default(),
            selected: Default::default(),
        };
    }
}
