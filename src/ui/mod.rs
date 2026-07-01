mod anim;
mod game;
mod render;

use engine::{
    board::{BOARD_HEIGHT, BOARD_WIDTH, Board, Move, Player, Position},
    engine::UCCIEngine,
    search::SearchState,
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

pub const CHESS_SIZE: usize = 57;
pub const CHESS_BOARD_WIDTH: i32 = 521;
pub const CHESS_BOARD_HEIGHT: i32 = 577;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Click(i32, i32),
    Undo,
    AIMove(Move),
    NewGame(Player),
    AnimTick,
}

pub struct AnimState {
    pub mv: Move,
    pub progress: f64,
}

pub struct ChessApp {
    pub game: Board,
    pub ui_search: SearchState,
    pub engine: UCCIEngine,
    pub ai_thinking: Arc<Mutex<bool>>,
    pub human_side: Arc<Mutex<Player>>,
    pub sender: app::Sender<Message>,
    pub receiver: app::Receiver<Message>,
    pub board_frame: Frame,
    pub status_label: Frame,
    pub cap_red_label: Frame,
    pub cap_black_label: Frame,
    pub anim: Option<AnimState>,
    pub board_img: SharedImage,
}

pub fn cell_top_left(pos: Position) -> (i32, i32) {
    let x = (pos.col + 1) * CHESS_SIZE as i32 - CHESS_SIZE as i32 / 2 - 24;
    let y = (pos.row + 1) * CHESS_SIZE as i32 - CHESS_SIZE as i32 / 2 - 24;
    (x, y)
}

impl ChessApp {
    fn new(game: Board, engine: UCCIEngine) -> Self {
        let (s, r) = app::channel::<Message>();
        let ai_thinking = Arc::new(Mutex::new(false));
        let human_side = Arc::new(Mutex::new(Player::Red));

        let board_img_data = include_bytes!("../../resources/board.jpg");
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
