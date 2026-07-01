/**
 * 引擎核心库入口
 *
 * 暴露子模块：board, constant, engine, search, zobrist
 * 以及开局库数据加载函数 book_data()
 */
pub mod board;
pub mod constant;
pub mod engine;
pub mod search;
pub mod zobrist;

pub fn book_data() -> &'static str {
    include_str!("../BOOK.DAT")
}
