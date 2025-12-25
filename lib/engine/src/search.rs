/*
 * Search State Module - AI搜索状态管理
 *
 * 将AI搜索相关的状态从Board中分离出来，实现：
 * - 更小的Board clone成本
 * - 更清晰的职责分离
 * - 零拷贝的开局库检查
 */

use crate::board::{Board, Chess, Move};
use crate::constant::{MAX, MAX_DEPTH, MIN, RECORD_SIZE};

/// 置换表记录的类型标志
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HashFlag {
    Exact, // 精确值
    Alpha, // 下界
    Beta,  // 上界
}

/// 置换表记录
#[derive(Clone, Copy, Debug)]
pub struct Record {
    pub zobrist_lock: u64,
    pub depth: i32,
    pub flag: HashFlag,
    pub best_move: Option<Move>,
    pub value: i32,
}

/// AI搜索状态
/// 包含所有搜索相关的临时数据，与Board的纯游戏状态分离
pub struct SearchState {
    /// 搜索节点计数器
    pub counter: i32,
    /// 走法生成计数器
    pub gen_counter: i32,
    /// 走法历史记录
    pub move_history: Vec<Move>,
    /// Zobrist哈希历史（用于重复局面检测）
    pub zobrist_history: Vec<u64>,
    /// 将军状态历史（用于长将检测）
    pub check_history: Vec<bool>,
    /// 上一层迭代的最佳走法序列
    pub best_moves_last: Vec<Move>,
    /// 置换表
    pub records: Vec<Option<Record>>,
    /// 当前搜索距离根节点的深度
    pub distance: i32,
    /// 杀手走法表：每层深度保存2个最佳走法
    pub killer_table: Vec<[Option<Move>; 2]>,
    /// 历史启发表：记录每个走法的历史得分
    /// 索引：from_square_index * 90 + to_square_index
    pub history_table: Vec<i32>,
}

impl SearchState {
    /// 创建新的搜索状态
    pub fn new() -> Self {
        SearchState {
            counter: 0,
            gen_counter: 0,
            move_history: vec![],
            zobrist_history: vec![],
            check_history: vec![],
            best_moves_last: vec![],
            records: vec![None; RECORD_SIZE as usize],
            distance: 0,
            killer_table: vec![[None, None]; MAX_DEPTH as usize],
            history_table: vec![0; 90 * 90],
        }
    }

    /// 重置搜索状态（用于新的搜索）
    pub fn reset(&mut self) {
        self.counter = 0;
        self.gen_counter = 0;
        self.move_history.clear();
        self.zobrist_history.clear();
        self.check_history.clear();
        self.best_moves_last.clear();
        self.records = vec![None; RECORD_SIZE as usize];
        self.distance = 0;
        self.killer_table = vec![[None, None]; MAX_DEPTH as usize];
        self.history_table = vec![0; 90 * 90];
    }

    pub fn push_move(&mut self, board: &mut Board, m: &Move) {
        self.zobrist_history.push(board.zobrist_value);
        // Turn is swapped AFTER apply_move in original logic,
        // but let's be careful. Original apply_move:
        // push old zobrist
        // apply physics
        // swap turn
        // push is_checked(new_turn)

        board.apply_move(m);
        self.check_history
            .push(board.is_checked(board.turn));

        self.move_history.push(m.clone());
        self.distance += 1;
    }

    pub fn pop_move(&mut self, board: &mut Board, m: &Move) {
        let prev_zobrist = self.zobrist_history.pop().unwrap_or(0);
        self.check_history.pop();
        self.move_history.pop();

        board.undo_move(m);
        board.zobrist_value = prev_zobrist;
        // Note: undo_move in board.rs already reverts turn and physical board
        // but it doesn't know about zobrist_history.
        self.distance -= 1;
    }

    pub fn push_null_move(&mut self, board: &mut Board) {
        self.zobrist_history.push(board.zobrist_value);
        board.do_null_move();
        self.distance += 1;
    }

    pub fn pop_null_move(&mut self, board: &mut Board) {
        self.zobrist_history.pop();
        board.undo_null_move();
        self.distance -= 1;
    }

    pub fn find_record(&self, board: &Board, alpha: i32, beta: i32, depth: i32) -> (Option<i32>, Option<Move>) {
        if let Some(record) = &self.records[(board.zobrist_value & (RECORD_SIZE - 1) as u64) as usize] {
            if record.zobrist_lock == board.zobrist_value_lock {
                let mut value = record.value;
                if value > 30000 {
                    value -= self.distance;
                } else if value < -30000 {
                    value += self.distance;
                }

                if record.depth >= depth {
                    match record.flag {
                        HashFlag::Exact => return (Some(value), record.best_move.clone()),
                        HashFlag::Alpha => {
                            if value <= alpha {
                                return (Some(value), record.best_move.clone());
                            }
                        }
                        HashFlag::Beta => {
                            if value >= beta {
                                return (Some(value), record.best_move.clone());
                            }
                        }
                    }
                }
                return (None, record.best_move.clone());
            }
        }
        (None, None)
    }

    pub fn add_record(&mut self, board: &Board, depth: i32, mut value: i32, flag: HashFlag, best_move: Option<Move>) {
        if value > 30000 {
            value += self.distance;
        } else if value < -30000 {
            value -= self.distance;
        }

        let index = (board.zobrist_value & (RECORD_SIZE - 1) as u64) as usize;
        if let Some(old_record) = &self.records[index] {
            if old_record.depth > depth {
                return;
            }
        }
        self.records[index] = Some(Record {
            value,
            depth,
            flag,
            best_move,
            zobrist_lock: board.zobrist_value_lock,
        });
    }

    fn history_index(&self, mv: &Move) -> usize {
        let from_idx = (mv.from.row * 9 + mv.from.col) as usize;
        let to_idx = (mv.to.row * 9 + mv.to.col) as usize;
        from_idx * 90 + to_idx
    }

    fn update_killer_move(&mut self, mv: &Move, depth: usize) {
        if depth >= self.killer_table.len() {
            return;
        }
        if let Some(killer1) = &self.killer_table[depth][0] {
            if killer1 != mv {
                self.killer_table[depth][1] = self.killer_table[depth][0].clone();
                self.killer_table[depth][0] = Some(mv.clone());
            }
        } else {
            self.killer_table[depth][0] = Some(mv.clone());
        }
    }

    fn update_history(&mut self, mv: &Move, depth: i32) {
        let idx = self.history_index(mv);
        if idx < self.history_table.len() {
            self.history_table[idx] += depth * depth;
        }
    }

    fn get_history_score(&self, mv: &Move) -> i32 {
        let idx = self.history_index(mv);
        if idx < self.history_table.len() {
            self.history_table[idx]
        } else {
            0
        }
    }

    fn sort_moves(&self, moves: &mut Vec<Move>, hash_move: Option<&Move>) {
        let depth = self.distance as usize;
        let killer1 = if depth < self.killer_table.len() {
            self.killer_table[depth][0].as_ref()
        } else {
            None
        };
        let killer2 = if depth < self.killer_table.len() {
            self.killer_table[depth][1].as_ref()
        } else {
            None
        };

        moves.sort_unstable_by_key(|mv| {
            if let Some(hm) = hash_move {
                if mv == hm {
                    return i32::MIN;
                }
            }

            if let Some(k1) = killer1 {
                if mv == k1 {
                    return i32::MIN + 1;
                }
            }
            if let Some(k2) = killer2 {
                if mv == k2 {
                    return i32::MIN + 2;
                }
            }

            let mut score = 0;
            if mv.capture != Chess::None {
                score += mv.capture.material_value() * 10 - mv.chess.material_value();
            }
            score += self.get_history_score(mv);
            -score
        });
    }
    pub fn alpha_beta_pvs(&mut self, board: &mut Board, depth: i32, alpha: i32, beta: i32) -> (i32, Option<Move>) {
        self.alpha_beta_pvs_internal(board, depth, alpha, beta, true)
    }

    fn alpha_beta_pvs_internal(
        &mut self,
        board: &mut Board,
        depth: i32,
        mut alpha: i32,
        beta: i32,
        allow_null: bool,
    ) -> (i32, Option<Move>) {
        let (tt_value, hash_move) = self.find_record(board, alpha, beta, depth);
        if let Some(v) = tt_value {
            return (v, hash_move);
        }

        if self.distance > 0 {
            let rep = self.rep_status(board, 1);
            if rep > 0 {
                return (self.rep_value(rep), None);
            }
        }

        if depth == 0 {
            self.counter += 1;
            return (self.quies(board, alpha, beta), None);
        }

        const NULL_MOVE_REDUCTION: i32 = 2;
        if allow_null && depth >= 3 && !board.is_checked(board.turn) && self.null_move_okay(board) {
            self.push_null_move(board);
            let (v, _) = self.alpha_beta_pvs_internal(board, depth - NULL_MOVE_REDUCTION - 1, -beta, -beta + 1, false);
            self.pop_null_move(board);
            if -v >= beta {
                return (beta, None);
            }
        }

        let mut hash_move_searched = false;
        if let Some(hm) = hash_move.as_ref() {
            if board.is_valid_move(hm) {
                self.push_move(board, hm);
                if !board.is_checked(board.turn.next()) {
                    hash_move_searched = true;
                    let v = -self
                        .alpha_beta_pvs_internal(board, depth - 1, -beta, -alpha, true)
                        .0;
                    self.pop_move(board, hm);

                    if v >= beta {
                        self.update_killer_move(hm, self.distance as usize);
                        self.update_history(hm, depth);
                        self.add_record(board, depth, v, HashFlag::Beta, Some(hm.clone()));
                        return (v, Some(hm.clone()));
                    }
                    if v > alpha {
                        alpha = v;
                    }
                } else {
                    self.pop_move(board, hm);
                }
            }
        }

        let mut count = 0;
        let mut moves = board.generate_move(false);
        self.sort_moves(&mut moves, None);

        let mut best_move = if hash_move_searched { hash_move.clone() } else { None };
        let mut best_value = if best_move.is_some() && alpha > MIN { alpha } else { MIN };

        for m in moves {
            if hash_move_searched {
                if let Some(hm) = hash_move.as_ref() {
                    if &m == hm {
                        continue;
                    }
                }
            }

            self.push_move(board, &m);
            if board.is_checked(board.turn.next()) {
                self.pop_move(board, &m);
                continue;
            }
            count += 1;

            let new_depth = if board.is_checked(board.turn) { depth } else { depth - 1 };

            let v = if count == 1 && !hash_move_searched {
                -self
                    .alpha_beta_pvs_internal(board, new_depth, -beta, -alpha, true)
                    .0
            } else {
                let scout = -self
                    .alpha_beta_pvs_internal(board, new_depth, -(alpha + 1), -alpha, false)
                    .0;
                if scout > alpha && scout < beta {
                    -self
                        .alpha_beta_pvs_internal(board, new_depth, -beta, -alpha, true)
                        .0
                } else {
                    scout
                }
            };

            self.pop_move(board, &m);

            if v > best_value {
                best_value = v;
                if v >= beta {
                    self.update_killer_move(&m, self.distance as usize);
                    self.update_history(&m, depth);
                    self.add_record(board, depth, v, HashFlag::Beta, Some(m.clone()));
                    return (v, Some(m.clone()));
                }
                if v > alpha {
                    alpha = v;
                    best_move = Some(m.clone());
                }
            }
        }

        if count == 0 {
            return (MIN + self.distance, None);
        }

        let hash_flag = if best_value > alpha {
            HashFlag::Exact
        } else {
            HashFlag::Alpha
        };
        self.add_record(board, depth, best_value, hash_flag, best_move.clone());
        (best_value, best_move)
    }

    pub fn rep_status(&self, board: &Board, mut recur: i32) -> i32 {
        if self.move_history.is_empty() {
            return 0;
        }
        let mut self_side = false;
        let mut perp_check = true;
        let mut opp_perp_check = true;

        let len = self.move_history.len();
        for i in (0..len).rev() {
            let m = &self.move_history[i];
            if m.capture != Chess::None {
                break;
            }

            if self_side {
                if i < self.check_history.len() {
                    perp_check &= self.check_history[i];
                }
                if i < self.zobrist_history.len() && self.zobrist_history[i] == board.zobrist_value {
                    recur -= 1;
                    if recur <= 0 {
                        return 1 + (if perp_check { 2 } else { 0 }) + (if opp_perp_check { 4 } else { 0 });
                    }
                }
            } else {
                if i < self.check_history.len() {
                    opp_perp_check &= self.check_history[i];
                }
            }
            self_side = !self_side;
        }
        0
    }

    pub fn rep_value(&self, rep_status: i32) -> i32 {
        const BAN_VAL: i32 = 30000 - 100;
        let val_loss = -BAN_VAL + self.distance;
        let val_win = BAN_VAL - self.distance;

        if (rep_status & 2) != 0 {
            return val_loss;
        }
        if (rep_status & 4) != 0 {
            return val_win;
        }
        if (self.distance & 1) == 0 { -20 } else { 20 }
    }

    pub fn quies(&mut self, board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        if self.distance > MAX_DEPTH {
            return board.evaluate(board.turn);
        }
        let v = board.evaluate(board.turn);
        if v >= beta {
            return beta;
        }
        if v > alpha {
            alpha = v;
        }

        let mut moves = if board.is_checked(board.turn.next()) {
            board.generate_move(false)
        } else {
            board.generate_move(true)
        };
        self.sort_moves(&mut moves, None);
        for m in moves {
            self.push_move(board, &m);
            if board.is_checked(board.turn.next()) {
                self.pop_move(board, &m);
                continue;
            }
            let v = -self.quies(board, -beta, -alpha);
            self.pop_move(board, &m);
            if v >= beta {
                return beta;
            }
            if v > alpha {
                alpha = v;
            }
        }
        alpha
    }

    pub fn iterative_deepening(&mut self, board: &mut Board, max_depth: i32) -> (i32, Option<Move>) {
        let mut best_move = None;
        let mut best_value = 0;
        for depth in 1..max_depth + 1 {
            let (v, bm) = self.alpha_beta_pvs(board, depth, MIN, MAX);
            if bm.is_some() {
                best_value = v;
                best_move = bm;
            }
            println!("depth {}: score {}, move {:?}", depth, v, best_move);
        }
        (best_value, best_move)
    }

    fn null_move_okay(&self, board: &Board) -> bool {
        board.get_player_score(board.turn) > 200
    }
}
