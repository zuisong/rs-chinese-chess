use engine::{
    board::{BOARD_HEIGHT, BOARD_WIDTH, Board, Move, Player, Position},
    engine::UCCIEngine,
};
use fltk::{
    app,
    button::Button,
    enums::*,
    frame::Frame,
    group::*,
    image::{JpegImage, SharedImage},
    prelude::*,
    window::*,
};
use std::sync::{Arc, Mutex};

const CHESS_SIZE: usize = 57;
const CHESS_BOARD_WIDTH: i32 = 521;
const CHESS_BOARD_HEIGHT: i32 = 577;

pub fn ui(mut game: Board, mut engine: UCCIEngine) -> anyhow::Result<()> {
    let app = app::App::default();
    let pand = 1;
    let mut top_window = Window::new(
        100,
        100,
        CHESS_BOARD_WIDTH + 120,
        CHESS_BOARD_HEIGHT + pand * 2,
        "中国象棋",
    );

    let mut chess_window = Window::default()
        .with_pos(pand, pand)
        .with_size(CHESS_BOARD_WIDTH + 120, CHESS_BOARD_HEIGHT);

    enum Message {
        Click(i32, i32),
        Undo,
        AIMove(Move), // AI 计算完成，返回走法
    }

    let (s, r) = app::channel::<Message>();

    // AI 是否正在思考
    let ai_thinking = Arc::new(Mutex::new(false));

    {
        // 画棋盘
        let data = include_bytes!("../resources/board.jpg");
        let mut background = SharedImage::from_image(&JpegImage::from_data(data)?)?;
        Frame::new(0, 0, CHESS_BOARD_WIDTH, CHESS_BOARD_HEIGHT, "")
            .draw(move |f| background.draw(f.x(), f.y(), f.width(), f.height()));
    }

    let mut flex = Flex::default_fill();

    let mut group = Group::default_fill();
    flex.fixed(&group, CHESS_BOARD_WIDTH);

    fn redrawn(group: &mut Group, game: &Board) {
        for x in 0..BOARD_WIDTH as usize {
            for y in 0..BOARD_HEIGHT as usize {
                let chess = game.chesses[y][x];

                let title = match chess.chess_type() {
                    Some(t) => t.name_value(),
                    None => continue,
                };

                let selected_chess = game.select_pos == (x as i32, y as i32).into();

                let x = (x + 1) * CHESS_SIZE - CHESS_SIZE / 2 - 24;
                let y = (y + 1) * CHESS_SIZE - CHESS_SIZE / 2 - 24;
                let padding = 4;
                let mut button = Button::new(
                    (x + padding) as i32,
                    (y + padding) as i32,
                    (CHESS_SIZE - 2 * padding) as i32,
                    (CHESS_SIZE - 2 * padding) as i32,
                    title,
                );
                button.set_label_color(if let Some(Player::Red) = chess.player() {
                    Color::Red
                } else {
                    Color::Blue
                });

                button.set_label_size((CHESS_SIZE * 6 / 10) as i32);
                button.set_frame(FrameType::RoundedBox);
                button.set_selection_color(Color::DarkBlue);
                button.set_color(Color::White);
                if selected_chess {
                    button.set_color(Color::Black);
                }
                group.add(&button);
            }
        }
    }

    redrawn(&mut group, &game);

    chess_window.handle({
        move |_, event| {
            if let Event::Push = event {
                let (click_x, click_y) = app::event_coords();
                if click_x > CHESS_BOARD_WIDTH {
                    return false; // Let button callbacks handle it
                }
                let (x, y) = (click_x / CHESS_SIZE as i32, click_y / CHESS_SIZE as i32);
                s.send(Message::Click(x, y));
                return true;
            }
            false
        }
    });

    let mut vpack = Pack::default_fill().with_type(PackType::Vertical);
    vpack.set_spacing(10);
    flex.add(&vpack);

    let mut undo_button = Button::default().with_label("悔棋");
    undo_button.set_callback({
        move |_| {
            s.send(Message::Undo);
        }
    });
    vpack.add(&undo_button);

    Button::default().with_label("功能");
    Button::default().with_label("功能");
    Button::default().with_label("功能");
    Button::default().with_label("功能");
    vpack.end();
    vpack.auto_layout();
    flex.fixed(&Group::default().with_size(10, 10), 10);
    flex.end();
    top_window.end();
    top_window.show();

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::Click(x, y) => {
                    let current_turn = game.turn;
                    // 检查是否 AI 正在思考
                    if *ai_thinking.lock().unwrap() {
                        println!("⏳ AI 正在思考中，请稍候...");
                        continue;
                    }

                    if current_turn == Player::Red {
                        let history_len_before = game.move_history.len();
                        game.click((x, y));
                        if game.move_history.len() > history_len_before {
                            // A move was made
                            let last_move = &game.move_history[game.move_history.len() - 1];
                            println!(
                                "👤 玩家走棋: {:?} 从 ({}, {}) 到 ({}, {})",
                                last_move.chess,
                                last_move.from.row,
                                last_move.from.col,
                                last_move.to.row,
                                last_move.to.col
                            );

                            group.clear();
                            chess_window.redraw();
                            redrawn(&mut group, &game);
                            app::flush();

                            // 同步检查开局库（避免昂贵的board clone）
                            engine.board = game.clone();
                            let sender = s.clone(); // Clone sender here to be available for both branches

                            if let Some(book_move) = engine.get_book_move() {
                                // 开局库有走法，直接使用
                                println!("📖 使用开局库走法");
                                sender.send(Message::AIMove(book_move));
                            } else {
                                // 需要搜索，启动后台线程
                                let mut board_for_search = engine.board.clone();
                                let thinking_flag = ai_thinking.clone();

                                *thinking_flag.lock().unwrap() = true;

                                rayon::spawn(move || {
                                    println!("🤔 AI 开始搜索...");
                                    let (_value, search_move) = board_for_search.iterative_deepening(6);

                                    // 释放思考标志
                                    *thinking_flag.lock().unwrap() = false;

                                    // 发送结果回主线程
                                    if let Some(m) = search_move {
                                        sender.send(Message::AIMove(m));
                                    }
                                });
                            }
                        }
                    }
                }
                Message::AIMove(ai_move) => {
                    println!("✅ AI 思考完成");
                    // 验证走法合法性
                    if game.is_move_legal(&ai_move) {
                        game.do_move(&ai_move);
                        group.clear();
                        chess_window.redraw();
                        redrawn(&mut group, &game);
                    } else {
                        println!("❌ AI 生成了非法走法，撤销玩家走法");
                        // 撤销玩家走法
                        if let Some(player_move) = game.move_history.last().cloned() {
                            game.undo_move(&player_move);
                            group.clear();
                            chess_window.redraw();
                            redrawn(&mut group, &game);
                        }
                    }
                }
                Message::Undo => {
                    if game.turn == Player::Red {
                        // A complete turn consists of the AI's move and the Player's move.
                        // We must undo both to return to the previous state.
                        if let Some(ai_move) = game.move_history.last().cloned() {
                            game.undo_move(&ai_move);
                        }
                        if let Some(player_move) = game.move_history.last().cloned() {
                            game.undo_move(&player_move);
                        }

                        game.select_pos = Position { row: -1, col: -1 }; // Reset selection

                        group.clear();
                        chess_window.redraw();
                        redrawn(&mut group, &game);
                    }
                }
            }
        }
    }
    Ok(())
}

trait BoardExt {
    fn click(&mut self, pos: (i32, i32));
    fn select(&mut self, pos: (i32, i32)) -> bool;
    fn move_to(
        &mut self,
        from: Position, // 起手位置
        to: Position,   // 落子位置
    );
}

impl BoardExt for Board {
    fn click(&mut self, pos: (i32, i32)) {
        let selected = self.select(pos);
        if !selected && self.chess_at(self.select_pos).player() == Some(self.turn) {
            self.move_to(self.select_pos, pos.into());
        }
    }

    fn select(&mut self, pos: (i32, i32)) -> bool {
        let chess = self.chess_at(pos.into());

        if chess.player() == Some(self.turn) {
            self.select_pos = pos.into();
            return true;
        }

        false
    }

    fn move_to(
        &mut self,
        from: Position, // 起手位置
        to: Position,   // 落子位置
    ) {
        let m = Move {
            player: self.turn,
            from,
            to,
            chess: self.chess_at(from),
            capture: self.chess_at(to),
        };
        if self.is_move_legal(&m) {
            self.do_move(&m);
        }
    }
}
