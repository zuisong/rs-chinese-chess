/* 引擎核心：对接棋盘与搜索，提供 UCCI 接口的核心能力 */
use crate::board::{Board, Move, Position};
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
    /// Ultra-lightweight FEN parser for opening book loading
    /// Returns (zobrist_value, zobrist_value_lock) without creating Board object
    fn parse_fen_for_zobrist(fen: &str) -> Option<(u64, u64)> {
        use crate::board::{Chess, Player};
        use crate::constant::{FEN_MAP, ZOBRIST_TABLE, ZOBRIST_TABLE_LOCK};

        let mut chesses = [[Chess::None; 9]; 10];
        let mut parts = fen.split(" ");
        let pos = parts.next()?;

        let mut i = 0;
        for row in pos.split("/") {
            let mut j = 0;
            for col in row.chars() {
                if col.is_numeric() {
                    j += col.to_digit(10)? as i32;
                } else {
                    if let Some(chess) = FEN_MAP.get(&col) {
                        if i < 10 && j < 9 {
                            chesses[i as usize][j as usize] = *chess;
                        }
                    }
                    j += 1;
                }
            }
            i += 1;
        }

        let turn = parts.next()?;
        let player = if turn == "b" { Player::Black } else { Player::Red };

        let zobrist_value = ZOBRIST_TABLE.calc_chesses(&chesses, player);
        let zobrist_value_lock = ZOBRIST_TABLE_LOCK.calc_chesses(&chesses, player);

        Some((zobrist_value, zobrist_value_lock))
    }
    pub fn new(book_data: Option<&str>) -> Self {
        let mut book = vec![];

        if let Some(data) = book_data {
            use std::time::Instant;

            let start = Instant::now();
            println!("⏳ 开始解析开局库...");

            // 使用简单迭代器处理每一行（已经足够快）
            book = data
                .lines()
                .map(|line| line.trim())
                .filter(|it| !it.is_empty())
                .filter_map(|line| {
                    let mut tokens = line.splitn(3, " ");
                    let m = tokens.next()?;
                    let weight = tokens.next()?;
                    let fen = tokens.next()?;

                    let (zobrist_value, zobrist_value_check) = Self::parse_fen_for_zobrist(fen)?;

                    Some(PreLoad {
                        zobrist_value,
                        zobrist_value_check,
                        best_move: m.to_owned(),
                        weight: weight.parse::<i32>().ok()?,
                    })
                })
                .collect();

            // 排序
            book.sort_unstable_by(|a, b| a.zobrist_value.cmp(&b.zobrist_value));

            let elapsed = start.elapsed();
            println!(
                "✅ 开局库加载完成，共加载{}个局面，耗时 {:.2}秒",
                book.len(),
                elapsed.as_secs_f64()
            );
        }

        UCCIEngine {
            board: Board::init(),
            book,
        }
    }
    pub fn search_in_book(&self) -> Option<String> {
        if self.book.is_empty() {
            return None;
        }

        let candidates = self
            .book
            .binary_search_by(|probe| probe.zobrist_value.cmp(&self.board.zobrist_value))
            .map(|i| &self.book[i])
            .into_iter()
            .filter(|x| x.zobrist_value_check == self.board.zobrist_value_lock)
            .collect::<Vec<&PreLoad>>();

        if candidates.len() > 0 {
            let mut buf = [0; 4];
            fastrand::fill(&mut buf);
            let index = i32::from_be_bytes(buf) % candidates.len() as i32;
            Some(candidates[index as usize].best_move.clone())
        } else {
            None
        }
    }

    /// Get opening book move for current board position
    /// Returns None if no book move is available
    pub fn get_book_move(&self) -> Option<Move> {
        self.search_in_book()
            .and_then(|move_str| self.parse_move_string(&move_str))
    }

    /// Parse a move string like "b0c2" into a Move struct
    fn parse_move_string(&self, move_str: &str) -> Option<Move> {
        if move_str.len() != 4 {
            return None;
        }

        let (from_str, to_str) = move_str.split_at(2);
        let from: Position = from_str.into();
        let to: Position = to_str.into();

        let m = Move {
            player: self.board.turn,
            from,
            to,
            chess: self.board.chess_at(from),
            capture: self.board.chess_at(to),
        };

        if self.board.is_move_legal(&m) { Some(m) } else { None }
    }

    /// Check if opening book has a move for current position
    pub fn has_book_move(&self) -> bool {
        self.search_in_book().is_some()
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
                for m_str in moves.as_str().split(" ") {
                    let (from_str, to_str) = m_str.split_at(2);
                    let from: Position = from_str.into();
                    let to: Position = to_str.into();
                    let m = Move {
                        player: self.board.turn,
                        from,
                        to,
                        chess: self.board.chess_at(from),
                        capture: self.board.chess_at(to),
                    };

                    // Only apply the move if it is legal
                    if self.board.is_move_legal(&m) {
                        self.board.apply_move(&m);
                    }
                }
            }
        }
    }

    // 执行搜索并输出最佳走子
    // 参数 depth: 搜索深度
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
