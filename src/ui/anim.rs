use engine::board::Move;
use fltk::app;

use super::{AnimState, ChessApp};

impl ChessApp {
    pub fn schedule_anim_tick(&self) {
        let sender = self.sender.clone();
        app::add_timeout3(0.016, move |_| {
            sender.send(super::Message::AnimTick);
        });
    }

    pub fn start_anim(&mut self, m: Move) {
        self.anim = Some(AnimState {
            mv: m,
            progress: 0.0,
        });
        self.schedule_anim_tick();
        self.redraw();
    }

    pub fn finalize_anim(&mut self) {
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
}
