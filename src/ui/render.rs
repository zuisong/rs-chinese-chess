use engine::board::{BOARD_HEIGHT, BOARD_WIDTH, Chess, Player, Position};
use fltk::{draw, enums::*, prelude::*};

use super::{CHESS_SIZE, ChessApp, cell_top_left};

impl ChessApp {
    pub fn redraw(&mut self) {
        let game = self.game.clone();
        let human_side = *self.human_side.lock().unwrap();
        let last_move = self.ui_search.move_history.last().cloned();
        let anim = self.anim.as_ref().map(|a| (a.mv, a.progress));
        let mut board_img = self.board_img.clone();

        self.board_frame.draw(move |f| {
            let flipped = human_side == Player::Black;

            board_img.draw(f.x(), f.y(), f.width(), f.height());

            Self::draw_last_move_highlight(last_move, flipped);
            Self::draw_selection_highlight(&game, flipped);
            Self::draw_all_pieces(&game, anim, flipped);
            Self::draw_animating_piece(anim, flipped);
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

    fn draw_last_move_highlight(last_move: Option<engine::board::Move>, flipped: bool) {
        if let Some(m) = last_move {
            let pos_from = if flipped { m.from.flip() } else { m.from };
            let pos_to = if flipped { m.to.flip() } else { m.to };

            let (fx, fy) = cell_top_left(pos_from);
            let (tx, ty) = cell_top_left(pos_to);

            draw::set_line_style(draw::LineStyle::Solid, 3);
            draw::set_draw_color(Color::Red);
            draw::draw_arc(
                fx + 5,
                fy + 5,
                CHESS_SIZE as i32 - 10,
                CHESS_SIZE as i32 - 10,
                0.0,
                360.0,
            );
            draw::draw_arc(
                tx + 5,
                ty + 5,
                CHESS_SIZE as i32 - 10,
                CHESS_SIZE as i32 - 10,
                0.0,
                360.0,
            );
            draw::set_line_style(draw::LineStyle::Solid, 1);
        }
    }

    fn draw_selection_highlight(game: &engine::board::Board, flipped: bool) {
        if game.select_pos.row != -1 {
            let pos = if flipped {
                game.select_pos.flip()
            } else {
                game.select_pos
            };
            let (sx, sy) = cell_top_left(pos);
            draw::set_line_style(draw::LineStyle::Solid, 4);
            draw::set_draw_color(Color::from_rgb(0, 200, 255));
            draw::draw_arc(
                sx,
                sy,
                CHESS_SIZE as i32 - 1,
                CHESS_SIZE as i32 - 1,
                0.0,
                360.0,
            );
            draw::set_line_style(draw::LineStyle::Solid, 1);
        }
    }

    fn draw_all_pieces(
        game: &engine::board::Board,
        anim: Option<(engine::board::Move, f64)>,
        flipped: bool,
    ) {
        for row in 0..BOARD_HEIGHT as usize {
            for col in 0..BOARD_WIDTH as usize {
                let chess = game.chesses[row][col];
                if chess == Chess::None {
                    continue;
                }

                if let Some((ref amv, _)) = anim {
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

                Self::draw_piece(display_pos, chess);
            }
        }
    }

    fn draw_animating_piece(anim: Option<(engine::board::Move, f64)>, flipped: bool) {
        if let Some((ref amv, progress)) = anim {
            let from_display = if flipped { amv.from.flip() } else { amv.from };
            let to_display = if flipped { amv.to.flip() } else { amv.to };
            let (from_x, from_y) = cell_top_left(from_display);
            let (to_x, to_y) = cell_top_left(to_display);

            let t = progress as f32;
            let px = (from_x as f32 + (to_x - from_x) as f32 * t) as i32;
            let py = (from_y as f32 + (to_y - from_y) as f32 * t) as i32;

            Self::draw_piece(Position::new(py, px), amv.chess);
        }
    }

    fn draw_piece(pos: Position, chess: Chess) {
        let (px, py) = cell_top_left(pos);
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
