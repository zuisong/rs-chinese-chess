use engine::{
    board::{BOARD_HEIGHT, BOARD_WIDTH, Board, Chess, Move, Player, Position},
    engine::UCCIEngine,
    search::SearchState,
};
use fltk::{
    app,
    button::Button,
    draw,
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

#[derive(Debug, Clone, Copy)]
enum Message {
    Click(i32, i32),
    Undo,
    AIMove(Move),    // AI 计算完成，返回走法
    NewGame(Player), // 重新开始，设置先手/后手
}

struct ChessApp {
    game: Board,
    ui_search: SearchState,
    engine: UCCIEngine,
    ai_thinking: Arc<Mutex<bool>>,
    human_side: Arc<Mutex<Player>>,
    sender: app::Sender<Message>,
    receiver: app::Receiver<Message>,
    board_frame: Frame,
    status_label: Frame,
}

impl ChessApp {
    fn new(game: Board, engine: UCCIEngine) -> Self {
        let (s, r) = app::channel::<Message>();
        let ai_thinking = Arc::new(Mutex::new(false));
        let human_side = Arc::new(Mutex::new(Player::Red));

        let mut top_window = Window::new(
            100,
            100,
            CHESS_BOARD_WIDTH + 160,
            CHESS_BOARD_HEIGHT + 2,
            "中国象棋 - 极美版",
        );

        let mut main_flex = Flex::default_fill().with_type(FlexType::Row);

        // --- 棋盘区 ---
        let mut board_frame = Frame::default().with_size(CHESS_BOARD_WIDTH, CHESS_BOARD_HEIGHT);
        main_flex.fixed(&board_frame, CHESS_BOARD_WIDTH);

        // --- 侧边栏 ---
        let mut sidebar = Pack::default().with_type(PackType::Vertical);
        sidebar.set_spacing(15);
        main_flex.add(&sidebar);

        // 顶部留白
        Frame::default().with_size(140, 20);

        let mut status_label = Frame::default()
            .with_size(140, 60)
            .with_label("等待开始...");
        status_label.set_label_size(18);
        status_label.set_label_color(Color::from_rgb(50, 50, 50));
        status_label.set_label_font(Font::HelveticaBoldItalic);

        let mut side_btn = Button::default()
            .with_size(120, 50)
            .with_label("执红 (先手)");
        side_btn.set_color(Color::from_rgb(245, 245, 245));
        side_btn.set_frame(FrameType::RoundedBox);
        side_btn.set_selection_color(Color::from_rgb(230, 230, 230));
        side_btn.set_label_size(16);
        side_btn.set_callback({
            let s = s.clone();
            let h = human_side.clone();
            move |b| {
                let mut side_lock = h.lock().unwrap();
                *side_lock = side_lock.next();
                let side = *side_lock;
                b.set_label(if side == Player::Red {
                    "执红 (先手)"
                } else {
                    "执黑 (后手)"
                });
                s.send(Message::NewGame(side));
            }
        });

        let mut restart_button = Button::default()
            .with_size(120, 50)
            .with_label("重新开始");
        restart_button.set_color(Color::from_rgb(220, 230, 255));
        restart_button.set_frame(FrameType::RoundedBox);
        restart_button.set_label_size(16);
        restart_button.set_callback({
            let s = s.clone();
            let h = human_side.clone();
            move |_| {
                let side = *h.lock().unwrap();
                s.send(Message::NewGame(side));
            }
        });

        let mut undo_button = Button::default()
            .with_size(120, 50)
            .with_label("悔棋回手");
        undo_button.set_color(Color::from_rgb(255, 235, 235));
        undo_button.set_frame(FrameType::RoundedBox);
        undo_button.set_label_size(16);
        undo_button.set_callback({
            let s = s.clone();
            move |_| s.send(Message::Undo)
        });

        sidebar.end();
        main_flex.end();
        top_window.end();
        top_window.show();

        board_frame.handle({
            let s = s.clone();
            let h = human_side.clone();
            move |_, event| {
                if let Event::Push = event {
                    let (click_x, click_y) = app::event_coords();
                    let (mut x, mut y) = (click_x / CHESS_SIZE as i32, click_y / CHESS_SIZE as i32);
                    if *h.lock().unwrap() == Player::Black {
                        x = BOARD_WIDTH - 1 - x;
                        y = BOARD_HEIGHT - 1 - y;
                    }
                    s.send(Message::Click(x, y));
                    return true;
                }
                false
            }
        });

        Self {
            game,
            ui_search: SearchState::new(),
            engine,
            ai_thinking,
            human_side,
            sender: s,
            receiver: r,
            board_frame,
            status_label,
        }
    }

    fn redraw(&mut self) {
        let game = self.game.clone();
        let human_side = *self.human_side.lock().unwrap();
        let last_move = self.ui_search.move_history.last().cloned();

        self.board_frame.draw(move |f| {
            let board_img_data = include_bytes!("../resources/board.jpg");
            let mut background = SharedImage::from_image(&JpegImage::from_data(board_img_data).unwrap()).unwrap();
            background.draw(f.x(), f.y(), f.width(), f.height());

            let flipped = human_side == Player::Black;

            // 1. 绘制上一步的高亮
            if let Some(m) = last_move {
                let pos_from = if flipped { m.from.flip() } else { m.from };
                let pos_to = if flipped { m.to.flip() } else { m.to };

                let highlight_color = Color::from_rgb(255, 255, 200); // Soft yellow highlight
                draw::set_draw_color(highlight_color);

                let draw_highlight = |pos: Position| {
                    let rx = (pos.col + 1) * CHESS_SIZE as i32 - CHESS_SIZE as i32 / 2 - 24;
                    let ry = (pos.row + 1) * CHESS_SIZE as i32 - CHESS_SIZE as i32 / 2 - 24;
                    draw::draw_rect_fill(
                        rx + 4,
                        ry + 4,
                        CHESS_SIZE as i32 - 8,
                        CHESS_SIZE as i32 - 8,
                        highlight_color,
                    );
                };
                draw_highlight(pos_from);
                draw_highlight(pos_to);
            }

            // 2. 绘制选中棋子的高亮
            if game.select_pos.row != -1 {
                let pos = if flipped {
                    game.select_pos.flip()
                } else {
                    game.select_pos
                };
                let rx = (pos.col + 1) * CHESS_SIZE as i32 - CHESS_SIZE as i32 / 2 - 24;
                let ry = (pos.row + 1) * CHESS_SIZE as i32 - CHESS_SIZE as i32 / 2 - 24;

                draw::set_line_style(draw::LineStyle::Solid, 4);
                draw::set_draw_color(Color::from_rgb(0, 200, 255)); // Deeper cyan
                draw::draw_arc(rx, ry, CHESS_SIZE as i32 - 1, CHESS_SIZE as i32 - 1, 0.0, 360.0);
                draw::set_line_style(draw::LineStyle::Solid, 1);
            }

            // 3. 绘制棋子
            for row in 0..BOARD_HEIGHT as usize {
                for col in 0..BOARD_WIDTH as usize {
                    let chess = game.chesses[row][col];
                    if chess == Chess::None {
                        continue;
                    }

                    let (display_row, display_col) = if flipped {
                        (BOARD_HEIGHT as usize - 1 - row, BOARD_WIDTH as usize - 1 - col)
                    } else {
                        (row, col)
                    };

                    let x = (display_col + 1) * CHESS_SIZE - CHESS_SIZE / 2 - 24;
                    let y = (display_row + 1) * CHESS_SIZE - CHESS_SIZE / 2 - 24;
                    let radius = (CHESS_SIZE / 2 - 4) as i32;
                    let cx = x as i32 + radius + 4;
                    let cy = y as i32 + radius + 4;

                    // 绘制棋子底色（白色圆圈）
                    draw::set_draw_color(Color::White);
                    draw::draw_pie(cx - radius, cy - radius, radius * 2, radius * 2, 0.0, 360.0);

                    // 绘制外边框（黑色圆圈）
                    draw::set_line_style(draw::LineStyle::Solid, 1);
                    draw::set_draw_color(Color::Black);
                    draw::draw_arc(cx - radius, cy - radius, radius * 2, radius * 2, 0.0, 360.0);

                    // 绘制文字
                    if let Some(ct) = chess.chess_type() {
                        let label = ct.name_value();
                        let text_color = if let Some(Player::Red) = chess.player() {
                            Color::Red
                        } else {
                            Color::Blue
                        };
                        draw::set_draw_color(text_color);
                        draw::set_font(Font::HelveticaBold, 24);
                        draw::draw_text2(label, cx - radius, cy - radius, radius * 2, radius * 2, Align::Center);
                    }
                }
            }
        });
        self.board_frame.redraw();

        // 更新状态标签
        let is_thinking = *self.ai_thinking.lock().unwrap();
        let label = if is_thinking {
            "AI 思考中...".to_string()
        } else {
            let turn_str = if self.game.turn == Player::Red {
                "红方走"
            } else {
                "黑方走"
            };
            format!("轮到: {}", turn_str)
        };
        self.status_label.set_label(&label);
        self.status_label.redraw();
    }

    fn handle_click(&mut self, x: i32, y: i32) {
        if *self.ai_thinking.lock().unwrap() {
            println!("⏳ AI 正在思考中，请稍候...");
            return;
        }

        let side = *self.human_side.lock().unwrap();
        if self.game.turn != side {
            return;
        }

        let history_len_before = self.ui_search.move_history.len();
        self.game.click(&mut self.ui_search, (x, y));

        // 无论是否走棋，都要重绘（为了显示选中效果）
        self.redraw();

        if self.ui_search.move_history.len() > history_len_before {
            app::flush();
            // 触发 AI
            self.trigger_ai();
        }
    }

    fn trigger_ai(&mut self) {
        self.engine.board = self.game.clone();
        let sender = self.sender.clone();

        if let Some(book_move) = self.engine.get_book_move() {
            println!("📖 使用开局库走法");
            sender.send(Message::AIMove(book_move));
        } else {
            let mut board_for_search = self.game.clone();
            let thinking_flag = self.ai_thinking.clone();
            *thinking_flag.lock().unwrap() = true;

            rayon::spawn(move || {
                println!("🤔 AI 开始搜索...");
                let mut search_state = SearchState::new();
                let (_value, search_move) = search_state.iterative_deepening(&mut board_for_search, 6);
                *thinking_flag.lock().unwrap() = false;
                if let Some(m) = search_move {
                    sender.send(Message::AIMove(m));
                }
            });
        }
    }

    fn run(&mut self) {
        self.redraw();
        while app::wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::Click(x, y) => self.handle_click(x, y),
                    Message::AIMove(ai_move) => {
                        println!("✅ AI 思考完成");
                        if self.game.is_move_legal(&ai_move) {
                            self.ui_search.push_move(&mut self.game, &ai_move);
                            self.redraw();
                        } else {
                            println!("❌ AI 生成了非法走法");
                        }
                    }
                    Message::Undo => {
                        let side = *self.human_side.lock().unwrap();
                        if self.game.turn == side {
                            if let Some(ai_move) = self.ui_search.move_history.last().cloned() {
                                self.ui_search.pop_move(&mut self.game, &ai_move);
                            }
                            if let Some(player_move) = self.ui_search.move_history.last().cloned() {
                                self.ui_search
                                    .pop_move(&mut self.game, &player_move);
                            }
                            self.game.select_pos = Position { row: -1, col: -1 };
                            self.redraw();
                        }
                    }
                    Message::NewGame(side) => {
                        println!("🆕 开始新游戏，玩家方: {:?}", side);
                        self.game = Board::init();
                        self.ui_search = SearchState::new();
                        *self.human_side.lock().unwrap() = side;
                        self.redraw();
                        app::flush();
                        if side == Player::Black {
                            self.trigger_ai();
                        }
                    }
                }
            }
        }
    }
}

pub fn ui(game: Board, engine: UCCIEngine) -> anyhow::Result<()> {
    let mut chess_app = ChessApp::new(game, engine);
    chess_app.run();
    Ok(())
}

trait BoardExt {
    fn click(&mut self, search: &mut SearchState, pos: (i32, i32));
    fn select(&mut self, pos: (i32, i32)) -> bool;
    fn move_to(
        &mut self,
        search: &mut SearchState,
        from: Position, // 起手位置
        to: Position,   // 落子位置
    );
}

impl BoardExt for Board {
    fn click(&mut self, search: &mut SearchState, pos: (i32, i32)) {
        let selected = self.select(pos);
        if !selected && self.chess_at(self.select_pos).player() == Some(self.turn) {
            self.move_to(search, self.select_pos, pos.into());
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
        search: &mut SearchState,
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
            search.push_move(self, &m);
        }
    }
}
