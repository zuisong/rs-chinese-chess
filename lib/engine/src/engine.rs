use crate::board::{Board, Move};
use getrandom::getrandom;
use regex::Regex;
use std::io;

#[derive(Debug)]
pub struct PreLoad {
    zobrist_value: u64,
    zobrist_value_check: u64,
    best_move: String,
    weight: i32,
}

// UCCI引擎
pub struct UCCIEngine {
    pub board: Board,
    pub book: Vec<PreLoad>,
}

impl UCCIEngine {
    pub fn new(book_data: Option<&str>) -> Self {
        let mut book = vec![];
        if let Some(data) = book_data {
            for line in data.split("\n") {
                if line.len() == 0 {
                    continue;
                }
                let mut tokens = line.splitn(3, " ");
                let m = tokens.next().unwrap();
                let weight = tokens.next().unwrap();
                let fen = tokens.next().unwrap();
                let board = Board::from_fen(fen);
                book.push(PreLoad {
                    zobrist_value: board.zobrist_value,
                    zobrist_value_check: board.zobrist_value_lock,
                    best_move: m.to_owned(),
                    weight: weight.parse::<i32>().unwrap(),
                });
            }
            book.sort_by(|a, b| a.zobrist_value.cmp(&b.zobrist_value));
            println!("加载开局库完成，共加载{}个局面", book.len());
            println!("{:?}", book[1000]);
        }
        UCCIEngine {
            board: Board::init(),
            book,
        }
    }
    pub fn search_in_book(&self) -> Option<String> {
        let candidates = self
            .book
            .binary_search_by(|probe| probe.zobrist_value.cmp(&self.board.zobrist_value))
            .map(|i| &self.book[i])
            .into_iter()
            .filter(|x| x.zobrist_value_check == self.board.zobrist_value_lock)
            .collect::<Vec<&PreLoad>>();
        if candidates.len() > 0 {
            let mut buf = [0; 4];
            getrandom(&mut buf).unwrap();
            let index = i32::from_be_bytes(buf) % candidates.len() as i32;
            Some(candidates[index as usize].best_move.clone())
        } else {
            None
        }
    }

    pub fn start(&mut self) {
        loop {
            let mut cmd = String::new();
            io::stdin().read_line(&mut cmd).unwrap();
            cmd = cmd.replace("\n", "");
            if cmd == "quit" {
                break;
            }
            let mut token = cmd.splitn(2, " ");
            let cmd = token.next().unwrap();
            match cmd {
                "ucci" => self.info(),
                "isready" => self.is_ready(),
                "position" => self.position(token.next().unwrap()),
                "go" => {
                    self.go(token
                        .next()
                        .unwrap()
                        .split(" ")
                        .last()
                        .unwrap()
                        .parse()
                        .unwrap());
                }
                _ => println!("not support"),
            }
        }
    }

    pub fn info(&self) {
        println!("id name nchess 1.0");
        println!("id copyright 2021-2022 www.nealian.cn");
        println!("id author nealian");
        println!("id user 2021-2022 www.nealian.cn");
        println!("option usemillisec type check");
        println!("ucciok");
    }

    pub fn is_ready(&self) {
        println!("readyok");
    }

    pub fn position(&mut self, param: &str) {
        let regex = Regex::new(
            r#"^(?:fen (?P<fen>[kabnrcpKABNRCP1-9/]+ [wrb] - - \d+ \d+)|(?P<startpos>startpos))(?: moves (?P<moves>[a-i]\d[a-i]\d(?: [a-i]\d[a-i]\d)*))?$"#,
        ).unwrap();
        for captures in regex.captures_iter(param) {
            if let Some(fen) = captures.name("fen") {
                self.board = Board::from_fen(fen.as_str());
            }
            if let Some(_) = captures.name("startpos") {
                self.board = Board::init();
            }
            if let Some(moves) = captures.name("moves") {
                for m in moves.as_str().split(" ") {
                    let (from, to) = m.split_at(2);
                    self.board.apply_move(&Move {
                        player: self.board.turn,
                        from: from.into(),
                        to: to.into(),
                        chess: self.board.chess_at(from.into()),
                        capture: self.board.chess_at(to.into()),
                    });
                }
            }
        }
    }

    pub fn go(&mut self, depth: i32) {
        if let Some(m) = self.search_in_book() {
            println!("bestmove {}", m);
            return;
        }
        let (value, best_move) = self.board.iterative_deepening(depth);
        if let Some(m) = best_move {
            if m.is_valid() {
                println!("bestmove {}{} value {}", m.from.to_string(), m.to.to_string(), value);
                return;
            }
        }
        println!("nobestmove");
    }
    pub fn quit() {
        println!("bye");
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::UCCIEngine;

    #[test]
    fn test_ucci_engine() {
        let mut engine = UCCIEngine::new(None);
        engine.info();
        engine.is_ready();
        engine.position(
        "fen rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1 moves b2d2 b9a7 a9a8 h7h0 b0a2 a8d8 a0b0 d8d2 b0b7 d2h2 b7g7 h9g7 g3g4 i9h9",
    );
        // engine.position("startpos moves b0c2");
        engine.go(4);
        println!("{:?}", engine.board.chesses);
        println!("{} {}", engine.board.gen_counter, engine.board.counter);
    }

    #[test]
    fn test_kill() {
        let mut engine = UCCIEngine::new(None);
        engine.info();
        engine.is_ready();
        engine.position("fen 4k4/9/9/9/9/9/9/4p4/9/5K3 b - - 0 1");
        // engine.position("startpos moves b0c2");
        let moves = engine.board.generate_move(false);
        println!("{:?}", moves);
        println!("{:?}", engine.board.chesses);
        engine.go(8);
        println!("{} {}", engine.board.gen_counter, engine.board.counter);
    }
}
