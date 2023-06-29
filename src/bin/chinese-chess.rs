use fltk::{
    app,
    button::Button,
    draw,
    enums::*,
    prelude::*,
    window::*,
};

const CHESS_SIZE: i32 = 60;

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gleam);
    let mut wind = Window::new(
        100,
        100,
        CHESS_SIZE * 10 - CHESS_SIZE,
        CHESS_SIZE * 10,
        "中国象棋",
    );

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

    let mut game = game::ChineseChess::init();

    fn redrawn(w: &mut DoubleWindow, game: &game::ChineseChess) {
        for chess in game.chessmen.iter() {
            let x = chess.position.x * CHESS_SIZE;
            let y = chess.position.y * CHESS_SIZE;

            let mut button = Button::new(x, y, CHESS_SIZE, CHESS_SIZE, chess.name_str());
            button.set_label_color(if chess.color { Color::Red } else { Color::Blue });
            button.set_label_size((CHESS_SIZE * 6 / 10) as i32);
            w.add(&button);
        }
    }
    redrawn(&mut wind, &game);
    wind.handle(move |w, event| {
        match event {
            Event::Push => {
                let (x, y) = app::event_coords();
                // 点击棋盘
                game.click(&game::Position {
                    x: x / CHESS_SIZE,
                    y: y / CHESS_SIZE,
                });
                w.redraw();
                w.clear();
                redrawn(w, &game);
                return true;
            }
            _ => {}
        };
        return false;
    });

    wind.end();
    wind.show();

    app.run().unwrap();
}

mod game {

    #[derive(PartialEq, Debug, Clone)]
    pub struct Position {
        pub x: i32,
        pub y: i32,
    }

    #[derive(PartialEq, Debug)]
    enum ChessType {
        Car,
        Horse,
        Elephant,
        Advisor,
        King,
        Cannon,
        Soldier,
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

            match self.chess_type {
                ChessType::Car => {
                    // 车:直线移动,不能越过其他棋子
                    if self.position.x == pos.x || self.position.y == pos.y {
                        // 同一行或同一列
                        let mut x = self.position.x;
                        let mut y = self.position.y;
                        loop {
                            if x < pos.x {
                                x += 1;
                            } else if x > pos.x {
                                x -= 1;
                            }
                            if y < pos.y {
                                y += 1;
                            } else if y > pos.y {
                                y -= 1;
                            }
                            if x == pos.x && y == pos.y {
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
                ChessType::Horse => {
                    // 马:日字走法,可以越过其他棋子
                    let x1 = self.position.x;
                    let y1 = self.position.y;
                    let x2 = pos.x;
                    let y2 = pos.y;
                    (x1 - x2).abs() * (y1 - y2).abs() == 2
                }
                ChessType::Elephant => {
                    // 象:斜线移动,不能越过其他棋子
                    let x1 = self.position.x;
                    let y1 = self.position.y;
                    let x2 = pos.x;
                    let y2 = pos.y;
                    (x1 - x2).abs() == 2 && (y1 - y2).abs() == 2
                }
                ChessType::Advisor => {
                    // 士:斜线移动,不能越过其他棋子
                    let x1 = self.position.x;
                    let y1 = self.position.y;
                    let x2 = pos.x;
                    let y2 = pos.y;
                    (x1 - x2).abs() == 1 && (y1 - y2).abs() == 1 && self.in_nine_palace(x2, y2)
                }
                ChessType::King => {
                    // 帅:一步一格,不能越过其他棋子
                    let x1 = self.position.x;
                    let y1 = self.position.y;
                    let x2 = pos.x;
                    let y2 = pos.y;
                    (x1 - x2).abs() + (y1 - y2).abs() == 1 && self.in_nine_palace(x2, y2)
                }
                ChessType::Cannon => {
                    // 炮:直线移动,可以越过其他棋子,但必须是吃子
                    if self.position.x == pos.x || self.position.y == pos.y {
                        // 同一行或同一列
                        let mut x = self.position.x;
                        let mut y = self.position.y;
                        let mut skipped = false;
                        loop {
                            if x < pos.x {
                                x += 1;
                            } else if x > pos.x {
                                x -= 1;
                            }
                            if y < pos.y {
                                y += 1;
                            } else if y > pos.y {
                                y -= 1;
                            }
                            if x == pos.x && y == pos.y {
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
                ChessType::Soldier => {
                    // 兵:直线移动,不能越过其他棋子
                    let x1 = self.position.x;
                    let y1 = self.position.y;
                    let x2 = pos.x;
                    let y2 = pos.y;
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
                ChessType::Car => "车",
                ChessType::Horse => "马",
                ChessType::Elephant => "象",
                ChessType::Advisor => "士",
                ChessType::King => match self.color {
                    true => "帅",
                    false => "将",
                },
                ChessType::Cannon => "炮",
                ChessType::Soldier => match self.color {
                    true => "兵",
                    false => "卒",
                },
            }
        }

        fn from(name: &str, color: bool, posi: (i32, i32)) -> Chess {
            let chess_type = match name {
                "车" => ChessType::Car,
                "马" => ChessType::Horse,
                "象" => ChessType::Elephant,
                "士" => ChessType::Advisor,
                "帅" | "将" => ChessType::King,
                "炮" => ChessType::Cannon,
                "兵" | "卒" => ChessType::Soldier,
                _ => unreachable!("{}", name),
            };

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
        pub chessmen: Vec<Chess>,
        selected: Option<usize>,
        turn: bool, // 当前走棋方
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

        pub fn init() -> ChineseChess {
            let chessmen = vec![
                // 红方棋子
                Chess::from("车", true, (0, 0)),
                Chess::from("车", true, (8, 0)),
                Chess::from("马", true, (7, 0)),
                Chess::from("马", true, (1, 0)),
                Chess::from("象", true, (6, 0)),
                Chess::from("象", true, (2, 0)),
                Chess::from("士", true, (5, 0)),
                Chess::from("士", true, (3, 0)),
                Chess::from("帅", true, (4, 0)),
                Chess::from("炮", true, (1, 2)),
                Chess::from("炮", true, (7, 2)),
                Chess::from("兵", true, (6, 3)),
                Chess::from("兵", true, (4, 3)),
                Chess::from("兵", true, (2, 3)),
                Chess::from("兵", true, (0, 3)),
                Chess::from("兵", true, (8, 3)),
                // 黑方棋子
                Chess::from("车", false, (0, 9)),
                Chess::from("车", false, (8, 9)),
                Chess::from("马", false, (7, 9)),
                Chess::from("马", false, (1, 9)),
                Chess::from("象", false, (6, 9)),
                Chess::from("象", false, (2, 9)),
                Chess::from("士", false, (5, 9)),
                Chess::from("士", false, (3, 9)),
                Chess::from("将", false, (4, 9)),
                Chess::from("炮", false, (1, 7)),
                Chess::from("炮", false, (7, 7)),
                Chess::from("卒", false, (6, 6)),
                Chess::from("卒", false, (4, 6)),
                Chess::from("卒", false, (2, 6)),
                Chess::from("卒", false, (0, 6)),
                Chess::from("卒", false, (8, 6)),
            ];

            return ChineseChess {
                chessmen: chessmen,
                selected: None,
                turn: true,
            };
        }
    }
}
