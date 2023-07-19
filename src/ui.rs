use crate::game::{self, Turn};
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

const CHESS_SIZE: i32 = 57;
const CHESS_BOARD_WIDTH: i32 = 521;
const CHESS_BOARD_HEIGHT: i32 = 577;
pub fn ui(mut game: game::ChineseChess) -> anyhow::Result<()> {
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

    fn redrawn(group: &mut Group, game: &game::ChineseChess) {
        for chess in game.chessmen.iter() {
            let x = (chess.position.x + 1) * CHESS_SIZE - CHESS_SIZE / 2 - 24;
            let y = (chess.position.y + 1) * CHESS_SIZE - CHESS_SIZE / 2 - 24;
            let padding = 4;
            let mut button = Button::new(
                x + padding,
                y + padding,
                CHESS_SIZE - 2 * padding,
                CHESS_SIZE - 2 * padding,
                chess.name_str(),
            );
            button.set_label_color(if chess.turn == Turn::Red {
                Color::Red
            } else {
                Color::Blue
            });
            button.set_label_size(CHESS_SIZE * 6 / 10);
            button.set_frame(FrameType::RoundedBox);
            button.set_selection_color(Color::DarkBlue);
            button.set_color(Color::White);
            group.add(&button);
        }
    }

    redrawn(&mut group, &game);
    chess_window.handle(move |w, event| {
        if let Event::Push = event {
            let (click_x, click_y) = app::event_coords();
            let (x, y) = (click_x / CHESS_SIZE, click_y / CHESS_SIZE);
            dbg!(x, y);
            // 点击棋盘
            game.click(&game::Position { x, y });
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
