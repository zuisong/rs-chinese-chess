use engine::board::{Board, Player, BOARD_HEIGHT, BOARD_WIDTH, Move, Position};
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

const CHESS_SIZE: usize = 57;
const CHESS_BOARD_WIDTH: i32 = 521;
const CHESS_BOARD_HEIGHT: i32 = 577;

pub fn ui(mut game: Board) -> anyhow::Result<()> {
    let app = app::App::default().with_scheme(app::Scheme::Oxy);
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

    {
        // 画棋盘
        let data = include_bytes!("../resources/board.jpg");
        let mut background = SharedImage::from_image(JpegImage::from_data(data)?)?;
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

                if chess.chess_type().is_none() {
                    continue;
                }

                let x = (x + 1) * CHESS_SIZE - CHESS_SIZE / 2 - 24;
                let y = (y + 1) * CHESS_SIZE - CHESS_SIZE / 2 - 24;
                let padding = 4;
                let title = chess
                    .chess_type()
                    .unwrap()
                    .name_value();
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
                group.add(&button);
            }
        }
    }

    let mut prev_clicked = (1, 1);

    redrawn(&mut group, &game);
    chess_window.handle(move |w, event| {
        if let Event::Push = event {
            let (click_x, click_y) = app::event_coords();
            let (x, y) = (click_x / CHESS_SIZE as i32, click_y / CHESS_SIZE as i32);
            dbg!(x, y);
            // 点击棋盘
            game.click((x, y));
            group.clear();
            w.redraw();

            redrawn(&mut group, &game);
            return true;
        }
        return false;
    });
    let mut hpack = Pack::default_fill();
    flex.add(&hpack);
    hpack.set_type(PackType::Vertical);
    hpack.set_spacing(10);
    Button::default().with_label("悔棋");
    Button::default().with_label("功能");
    Button::default().with_label("功能");
    Button::default().with_label("功能");
    Button::default().with_label("功能");
    hpack.end();
    hpack.auto_layout();
    flex.fixed(&Group::default().with_size(10, 10), 10);
    flex.end();
    top_window.end();
    top_window.show();
    app.run().unwrap();
    Ok(())
}

trait BoardExt {
    fn click(&mut self, pos: &(i32, i32));
    fn select(&mut self, pos: &(i32, i32)) -> bool;
    fn move_to(&mut self,
               from: Position, // 起手位置
               to: Position,   // 落子位置
    );
}


impl BoardExt for Board {
    fn click(&mut self, pos: (i32, i32)) {
        let selected = self.select(pos.clone());
        if !selected {
            self.move_to(pos);
        }
    }

    fn select(&mut self, (x, y): (i32, i32)) -> bool {
        let chess = self.chesses[y as usize][x as usize];

        if chess.chess_type().is_none() {
            return false
        }
        return true
    }

    fn move_to(&mut self,
               from: Position, // 起手位置
               to: Position,   // 落子位置
    ) {
        self.do_move(&Move {
            player: self.turn,
            from: from.into(),
            to: to.into(),
            chess: self.chess_at(from.into()),
            capture: self.chess_at(to.into()),
        })
    }
}

