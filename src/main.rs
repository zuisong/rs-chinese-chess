use fltk::{app, button::Button, draw, enums::*, frame::Frame, group::*, prelude::*, window::*};
const CHESS_SIZE: i32 = 60;
fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gleam);
    let mut wind = Window::new(
        100,
        100,
        CHESS_SIZE * 10 - CHESS_SIZE + CHESS_SIZE * 2,
        CHESS_SIZE * 10,
        "中国象棋",
    );

    let mut flex = Flex::default_fill();

    // 画棋盘格
    wind.draw(move |_w| {
        draw::set_draw_color(Color::from_rgb(255, 255, 255));
        for i in 0..9 {
            draw::draw_line(
                CHESS_SIZE / 2 + i * CHESS_SIZE,
                CHESS_SIZE / 2,
                CHESS_SIZE / 2 + i * CHESS_SIZE,
                10 * CHESS_SIZE - CHESS_SIZE / 2,
            );
        }
        for i in 0..10 {
            draw::draw_line(
                CHESS_SIZE / 2,
                CHESS_SIZE / 2 + i * CHESS_SIZE,
                9 * CHESS_SIZE - CHESS_SIZE / 2,
                CHESS_SIZE / 2 + i * CHESS_SIZE,
            );
        }
    });

    let mut group = Group::default_fill();
    flex.fixed(&group, CHESS_SIZE * 10 - CHESS_SIZE);

    let mut game: game::ChineseChess = Default::default();
    fn redrawn(group: &mut Group, game: &game::ChineseChess) {
        for chess in game.chessmen.iter() {
            let x = (chess.position.x) * CHESS_SIZE;
            let y = (chess.position.y) * CHESS_SIZE;
            let padding = 4;
            let mut button = Button::new(
                x + padding,
                y + padding,
                CHESS_SIZE - 2 * padding,
                CHESS_SIZE - 2 * padding,
                chess.name_str(),
            );
            button.set_label_color(if chess.color { Color::Red } else { Color::Blue });
            button.set_label_size((CHESS_SIZE * 6 / 10) as i32);
            button.set_frame(FrameType::RoundedBox);
            button.set_selection_color(Color::DarkBlue);
            button.set_color(Color::White);
            group.add(&button);
        }
    }

    redrawn(&mut group, &game);
    wind.handle(move |w, event| {
        if let Event::Push = event {
            let (x, y) = app::event_coords();
            // 点击棋盘
            game.click(&game::Position {
                x: x / CHESS_SIZE,
                y: y / CHESS_SIZE,
            });
            w.redraw();

            group.clear();
            redrawn(&mut group, &game);
            return true;
        }
        return false;
    });
    let mut hpack = Pack::default_fill();
    hpack.set_type(PackType::Vertical);
    hpack.set_spacing(5);
    flex.add(&hpack);

    hpack.add(&Button::default().with_label("悔棋"));
    hpack.add(&Button::default().with_label("功能"));
    hpack.add(&Button::default().with_label("功能"));
    hpack.add(&Button::default().with_label("功能"));
    hpack.add(&Button::default().with_label("功能"));
    hpack.end();
    hpack.auto_layout();
    flex.fixed(&Group::default().with_size(10, 10), 10);
    flex.end();
    wind.end();
    wind.show();
    app.run().unwrap();
}
mod game {

    use std::fmt;
    use ChessType::*;

    #[derive(PartialEq, Debug, Clone)]
    pub struct Position {
        pub x: i32,
        pub y: i32,
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
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
        pub color: bool,
        pub position: Position,
    }
    impl Chess {
        fn can_move_to(&self, pos: &Position, game: &ChineseChess) -> bool {
            if let Some(chess) = game.get_chess(pos) {
                if chess.color == self.color {
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
                    if x1 == x2 || y1 == y2 {
                        // 同一行或同一列
                        let mut x = self.position.x;
                        let mut y = self.position.y;
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
                    false
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
                    // 士:斜线移动,不能越过其他棋子
                    (x1 - x2).abs() == 1 && (y1 - y2).abs() == 1 && self.in_nine_palace(x2, y2)
                }
                帅 => {
                    // 帅:一步一格,不能越过其他棋子
                    (x1 - x2).abs() + (y1 - y2).abs() == 1 && self.in_nine_palace(x2, y2)
                }
                炮 => {
                    // 炮:直线移动,可以越过其他棋子,但必须是吃子
                    if x1 == x2 || y1 == y2 {
                        // 同一行或同一列
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
                    false
                }
                兵 => {
                    // 兵:直线移动,不能越过其他棋子
                    if self.color && y1 < 5 || !self.color && y1 > 4 {
                        // 没过河,只能向前
                        x1 == x2 && (self.color && y2 == y1 + 1 || !self.color && y2 == y1 - 1)
                    } else {
                        // 过了河,可以左右
                        (x1 == x2 && (y2 == y1 + 1 || y2 == y1 - 1))
                            || (x2 == x1 + 1 || x2 == x1 - 1)
                    }
                }
            }
        }
        fn in_nine_palace(&self, x: i32, y: i32) -> bool {
            if self.color {
                // 红方,九宫格在底部两个区域
                (3..=5).contains(&x) && (0..=2).contains(&y)
            } else {
                // 蓝方,九宫格在顶部两个区域
                (3..=5).contains(&x) && (7..=9).contains(&y)
            }
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

    impl From<(ChessType, bool, (i32, i32))> for Chess {
        fn from(value: (ChessType, bool, (i32, i32))) -> Chess {
            let (chess_type, color, posi) = value;
            Chess {
                chess_type,
                color,
                position: Position {
                    x: posi.0,
                    y: posi.1,
                },
            }
        }
    }

    pub struct ChineseChess {
        pub chessmen: Vec<Chess>,                 // 棋盘上的棋子
        selected: Option<usize>,                  // 当前选中的棋子序号
        turn: bool,                               // 当前走棋方
        history: Vec<(bool, Position, Position)>, // 历史记录 方便撤回
    }
    impl ChineseChess {
        fn has_chess(&self, pos: &Position) -> bool {
            self.chessmen.iter().any(|c| c.position == *pos)
        }
        fn get_chess(&self, pos: &Position) -> Option<&Chess> {
            self.chessmen.iter().find(|c| c.position == *pos)
        }
        pub fn click(&mut self, pos: &Position) {
            let selected = self.select(pos);
            if !selected {
                self.move_to(pos);
            }
        }
        fn select(&mut self, pos: &Position) -> bool {
            for (i, chess) in self.chessmen.iter().enumerate() {
                if chess.position == *pos && chess.color == self.turn {
                    self.selected = Some(i);
                    return true;
                }
            }
            return false;
        }
        fn move_to(&mut self, pos: &Position) {
            // eat chess
            let mut eat_chess = None;
            for (i, chess) in self.chessmen.iter().enumerate() {
                if chess.position == *pos && chess.color != self.turn {
                    eat_chess = Some(i);
                }
            }
            // move to target
            if let Some(selected) = self.selected {
                let chess = &self.chessmen[selected];
                if chess.can_move_to(&pos, &self) {
                    self.history
                        .push((self.turn, chess.position.clone(), pos.clone()));
                    let chess = &mut self.chessmen[selected];
                    chess.position = pos.clone();
                    self.turn = !self.turn; // 改变走棋方
                    self.selected = None;
                    if let Some(idx) = eat_chess {
                        self.chessmen.remove(idx);
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
                // 红方棋子
                (车, true, (0, 0)),
                (车, true, (8, 0)),
                (马, true, (7, 0)),
                (马, true, (1, 0)),
                (象, true, (6, 0)),
                (象, true, (2, 0)),
                (士, true, (5, 0)),
                (士, true, (3, 0)),
                (帅, true, (4, 0)),
                (炮, true, (1, 2)),
                (炮, true, (7, 2)),
                (兵, true, (6, 3)),
                (兵, true, (4, 3)),
                (兵, true, (2, 3)),
                (兵, true, (0, 3)),
                (兵, true, (8, 3)),
                // 黑方棋子
                (车, false, (0, 9)),
                (车, false, (8, 9)),
                (马, false, (7, 9)),
                (马, false, (1, 9)),
                (象, false, (6, 9)),
                (象, false, (2, 9)),
                (士, false, (5, 9)),
                (士, false, (3, 9)),
                (帅, false, (4, 9)),
                (炮, false, (1, 7)),
                (炮, false, (7, 7)),
                (兵, false, (6, 6)),
                (兵, false, (4, 6)),
                (兵, false, (2, 6)),
                (兵, false, (0, 6)),
                (兵, false, (8, 6)),
            ]
            .into_iter()
            .map(Into::into)
            .collect();
            return ChineseChess {
                chessmen,
                turn: true,
                history: vec![],
                selected: None,
            };
        }
    }

    impl fmt::Display for ChessType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }
}
