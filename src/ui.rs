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
    AIMove(Move),
    NewGame(Player),
    AnimTick,
}

struct AnimState {
    mv: Move,
    progress: f64,
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
    cap_red_label: Frame,
    cap_black_label: Frame,
    anim: Option<AnimState>,
    board_img: SharedImage,
}

fn cell_top_left(pos: Position) -> (i32, i32) {
    let x = (pos.col + 1) * CHESS_SIZE as i32 - CHESS_SIZE as i32 / 2 - 24;
    let y = (pos.row + 1) * CHESS_SIZE as i32 - CHESS_SIZE as i32 / 2 - 24;
    (x, y)
}

impl ChessApp {
    fn new(game: Board, engine: UCCIEngine) -> Self {
        let (s, r) = app::channel::<Message>();
        let ai_thinking = Arc::new(Mutex::new(false));
        let human_side = Arc::new(Mutex::new(Player::Red));

        let board_img_data = include_bytes!("../resources/board.jpg");
        let board_img =
            SharedImage::from_image(&JpegImage::from_data(board_img_data).unwrap()).unwrap();

        let mut top_window = Window::new(
            100,
            100,
            CHESS_BOARD_WIDTH + 160,
            CHESS_BOARD_HEIGHT + 2,
            "中国象棋 - 极美版",
        );

        let mut main_flex = Flex::default_fill().with_type(FlexType::Row);

        let mut board_frame = Frame::default().with_size(CHESS_BOARD_WIDTH, CHESS_BOARD_HEIGHT);
        main_flex.fixed(&board_frame, CHESS_BOARD_WIDTH);

        let mut sidebar = Pack::default().with_type(PackType::Vertical);
        sidebar.set_spacing(8);

        Frame::default().with_size(140, 10);

        let mut status_label = Frame::default()
            .with_size(140, 45)
            .with_label("等待开始...");
        status_label.set_label_size(16);
        status_label.set_label_color(Color::from_rgb(50, 50, 50));
        status_label.set_label_font(Font::HelveticaBoldItalic);

        Frame::default().with_size(140, 2);

        let mut cap_header = Frame::default()
            .with_size(140, 16)
            .with_label("被吃棋子:");
        cap_header.set_label_size(12);
        cap_header.set_label_color(Color::from_rgb(100, 100, 100));
        cap_header.set_label_font(Font::HelveticaBold);

        let mut cap_red_label = Frame::default().with_size(140, 20);
        cap_red_label.set_label_size(12);
        cap_red_label.set_label_color(Color::from_rgb(180, 40, 40));
        cap_red_label.set_label_font(Font::Helvetica);

        let mut cap_black_label = Frame::default().with_size(140, 20);
        cap_black_label.set_label_size(12);
        cap_black_label.set_label_color(Color::from_rgb(40, 60, 180));
        cap_black_label.set_label_font(Font::Helvetica);

        Frame::default().with_size(140, 4);

        let mut side_btn = Button::default()
            .with_size(120, 40)
            .with_label("执红 (先手)");
        side_btn.set_color(Color::from_rgb(245, 245, 245));
        side_btn.set_frame(FrameType::RoundedBox);
        side_btn.set_label_size(14);
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
            .with_size(120, 40)
            .with_label("重新开始");
        restart_button.set_color(Color::from_rgb(220, 230, 255));
        restart_button.set_frame(FrameType::RoundedBox);
        restart_button.set_label_size(14);
        restart_button.set_callback({
            let s = s.clone();
            let h = human_side.clone();
            move |_| {
                let side = *h.lock().unwrap();
                s.send(Message::NewGame(side));
            }
        });

        let mut undo_button = Button::default()
            .with_size(120, 40)
            .with_label("悔棋回手");
        undo_button.set_color(Color::from_rgb(255, 235, 235));
        undo_button.set_frame(FrameType::RoundedBox);
        undo_button.set_label_size(14);
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
                    let (mut x, mut y) =
                        (click_x / CHESS_SIZE as i32, click_y / CHESS_SIZE as i32);
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
            cap_red_label,
            cap_black_label,
            anim: None,
            board_img,
        }
    }

    fn schedule_anim_tick(&self) {
        let sender = self.sender.clone();
        app::add_timeout3(0.016, move |_| {
            sender.send(Message::AnimTick);
        });
    }

    fn start_anim(&mut self, m: Move) {
        self.anim = Some(AnimState {
            mv: m,
            progress: 0.0,
        });
        self.schedule_anim_tick();
        self.redraw();
    }

    fn finalize_anim(&mut self) {
        if let Some(anim) = self.anim.take() {
            let is_human_move = anim.mv.player == *self.human_side.lock().unwrap();
            self.ui_search.push_move(&mut self.game, &anim.mv);
            self.redraw();
            if is_human_move {
                app::flush();
                self.trigger_ai();
            }
        }
    }

    fn update_captured_label(&mut self) {
        let mut red_lost = vec![];
        let mut black_lost = vec![];

        for m in &self.ui_search.move_history {
            match m.capture {
                Chess::Red(ct) => red_lost.push(ct),
                Chess::Black(ct) => black_lost.push(ct),
                _ => {}
            }
        }

        let red_text: String = red_lost
            .iter()
            .map(|ct| ct.name_value())
            .collect::<Vec<_>>()
            .join(" ");
        let black_text: String = black_lost
            .iter()
            .map(|ct| ct.name_value())
            .collect::<Vec<_>>()
            .join(" ");

        self.cap_red_label
            .set_label(&format!("红: {}", if red_text.is_empty() { "-" } else { &red_text }));
        self.cap_black_label.set_label(&format!(
            "黑: {}",
            if black_text.is_empty() { "-" } else { &black_text }
        ));
        self.cap_red_label.redraw();
        self.cap_black_label.redraw();
    }

    fn redraw(&mut self) {
        let game = self.game.clone();
        let human_side = *self.human_side.lock().unwrap();
        let last_move = self.ui_search.move_history.last().cloned();
        let anim = self.anim.as_ref().map(|a| (a.mv, a.progress));
        let mut board_img = self.board_img.clone();

        self.board_frame.draw(move |f| {
            let flipped = human_side == Player::Black;

            board_img.draw(f.x(), f.y(), f.width(), f.height());

            // 1. last move highlight
            if let Some(m) = last_move {
                let pos_from = if flipped { m.from.flip() } else { m.from };
                let pos_to = if flipped { m.to.flip() } else { m.to };

                let (fx, fy) = cell_top_left(pos_from);
                let (tx, ty) = cell_top_left(pos_to);

                draw::set_line_style(draw::LineStyle::Solid, 3);
                draw::set_draw_color(Color::Red);
                draw::draw_arc(fx + 5, fy + 5, CHESS_SIZE as i32 - 10, CHESS_SIZE as i32 - 10, 0.0, 360.0);
                draw::draw_arc(tx + 5, ty + 5, CHESS_SIZE as i32 - 10, CHESS_SIZE as i32 - 10, 0.0, 360.0);
                draw::set_line_style(draw::LineStyle::Solid, 1);
            }

            // 2. selection highlight
            if game.select_pos.row != -1 {
                let pos = if flipped {
                    game.select_pos.flip()
                } else {
                    game.select_pos
                };
                let (sx, sy) = cell_top_left(pos);
                draw::set_line_style(draw::LineStyle::Solid, 4);
                draw::set_draw_color(Color::from_rgb(0, 200, 255));
                draw::draw_arc(sx, sy, CHESS_SIZE as i32 - 1, CHESS_SIZE as i32 - 1, 0.0, 360.0);
                draw::set_line_style(draw::LineStyle::Solid, 1);
            }

            let anim_data = anim;

            // 3. draw all pieces (skip animating piece source)
            for row in 0..BOARD_HEIGHT as usize {
                for col in 0..BOARD_WIDTH as usize {
                    let chess = game.chesses[row][col];
                    if chess == Chess::None {
                        continue;
                    }

                    if let Some((ref amv, _)) = anim_data {
                        if Position::new(row as i32, col as i32) == amv.from {
                            continue;
                        }
                    }

                    let display_pos = if flipped {
                        Position::new(
                            BOARD_HEIGHT as i32 - 1 - row as i32,
                            BOARD_WIDTH as i32 - 1 - col as i32,
                        )
                    } else {
                        Position::new(row as i32, col as i32)
                    };

                    let (px, py) = cell_top_left(display_pos);
                    let radius = (CHESS_SIZE / 2 - 4) as i32;
                    let cx = px + radius + 4;
                    let cy = py + radius + 4;

                    draw::set_draw_color(Color::White);
                    draw::draw_pie(cx - radius, cy - radius, radius * 2, radius * 2, 0.0, 360.0);

                    draw::set_line_style(draw::LineStyle::Solid, 1);
                    draw::set_draw_color(Color::Black);
                    draw::draw_arc(cx - radius, cy - radius, radius * 2, radius * 2, 0.0, 360.0);

                    if let Some(ct) = chess.chess_type() {
                        let text_color = if let Some(Player::Red) = chess.player() {
                            Color::Red
                        } else {
                            Color::Blue
                        };
                        draw::set_draw_color(text_color);
                        draw::set_font(Font::HelveticaBold, 24);
                        draw::draw_text2(
                            ct.name_value(),
                            cx - radius,
                            cy - radius,
                            radius * 2,
                            radius * 2,
                            Align::Center,
                        );
                    }
                }
            }

            // 4. draw animating piece at interpolated position
            if let Some((ref amv, progress)) = anim_data {
                let from_display = if flipped { amv.from.flip() } else { amv.from };
                let to_display = if flipped { amv.to.flip() } else { amv.to };
                let (from_x, from_y) = cell_top_left(from_display);
                let (to_x, to_y) = cell_top_left(to_display);

                let t = progress as f32;
                let px = (from_x as f32 + (to_x - from_x) as f32 * t) as i32;
                let py = (from_y as f32 + (to_y - from_y) as f32 * t) as i32;
                let radius = (CHESS_SIZE / 2 - 4) as i32;
                let cx = px + radius + 4;
                let cy = py + radius + 4;

                if let Some(ct) = amv.chess.chess_type() {
                    let text_color = if let Some(Player::Red) = amv.chess.player() {
                        Color::Red
                    } else {
                        Color::Blue
                    };
                    draw::set_draw_color(text_color);
                    draw::set_font(Font::HelveticaBold, 24);
                    draw::draw_text2(
                        ct.name_value(),
                        cx - radius,
                        cy - radius,
                        radius * 2,
                        radius * 2,
                        Align::Center,
                    );
                }
            }
        });
        self.board_frame.redraw();

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

        self.update_captured_label();
    }

    fn handle_click(&mut self, x: i32, y: i32) {
        if *self.ai_thinking.lock().unwrap() || self.anim.is_some() {
            return;
        }

        let side = *self.human_side.lock().unwrap();
        if self.game.turn != side {
            return;
        }

        let pos = Position::new(y, x);
        let chess = self.game.chess_at(pos);

        if chess.player() == Some(self.game.turn) {
            self.game.select_pos = pos;
            self.redraw();
        } else if self.game.select_pos.row != -1 {
            let from = self.game.select_pos;
            let from_chess = self.game.chess_at(from);
            if from_chess.player() == Some(self.game.turn) {
                let m = Move {
                    player: self.game.turn,
                    from,
                    to: pos,
                    chess: from_chess,
                    capture: self.game.chess_at(pos),
                };
                if self.game.is_move_legal(&m) {
                    self.game.select_pos = Position { row: -1, col: -1 };
                    self.start_anim(m);
                    return;
                }
            }
            self.game.select_pos = Position { row: -1, col: -1 };
            self.redraw();
        }
    }

    fn trigger_ai(&mut self) {
        self.engine.board = self.game.clone();
        let sender = self.sender.clone();

        if let Some(book_move) = self.engine.get_book_move() {
            sender.send(Message::AIMove(book_move));
        } else {
            let mut board_for_search = self.game.clone();
            let thinking_flag = self.ai_thinking.clone();
            *thinking_flag.lock().unwrap() = true;

            rayon::spawn(move || {
                let mut search_state = SearchState::new();
                let (_value, search_move) =
                    search_state.iterative_deepening(&mut board_for_search, 6);
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
                        if self.game.is_move_legal(&ai_move) {
                            self.start_anim(ai_move);
                        }
                    }
                    Message::AnimTick => {
                        if let Some(ref mut anim) = self.anim {
                            anim.progress += 1.0 / 12.0;
                            if anim.progress >= 1.0 {
                                self.finalize_anim();
                            } else {
                                self.schedule_anim_tick();
                                self.redraw();
                            }
                        }
                    }
                    Message::Undo => {
                        if self.anim.is_some() {
                            continue;
                        }
                        let side = *self.human_side.lock().unwrap();
                        if self.game.turn == side {
                            if let Some(ai_move) = self.ui_search.move_history.last().cloned() {
                                self.ui_search.pop_move(&mut self.game, &ai_move);
                            }
                            if let Some(player_move) = self.ui_search.move_history.last().cloned() {
                                self.ui_search.pop_move(&mut self.game, &player_move);
                            }
                            self.game.select_pos = Position { row: -1, col: -1 };
                            self.redraw();
                        }
                    }
                    Message::NewGame(side) => {
                        self.game = Board::init();
                        self.ui_search = SearchState::new();
                        *self.human_side.lock().unwrap() = side;
                        self.anim = None;
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
