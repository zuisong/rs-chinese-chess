use engine::{
    board::{Chess, Move, Position},
    search::SearchState,
};
use fltk::prelude::*;

use super::ChessApp;

impl ChessApp {
    pub fn update_captured_label(&mut self) {
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

    pub fn handle_click(&mut self, x: i32, y: i32) {
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

    pub fn trigger_ai(&mut self) {
        self.engine.board = self.game.clone();
        let sender = self.sender.clone();

        if let Some(book_move) = self.engine.get_book_move() {
            sender.send(super::Message::AIMove(book_move));
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
                    sender.send(super::Message::AIMove(m));
                }
            });
        }
    }
}
