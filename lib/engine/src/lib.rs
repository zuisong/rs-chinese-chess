/**
 * 详细中文注释 - 引擎核心库入口
 *
 * 说明
 * - 暴露四个子模块：board, constant, engine, search, zobrist
 * - 这些模块共同构成象棋引擎的核心逻辑：棋盘状态、搜索算法、哈希、以及对外的引擎实现
 */
pub mod board;
pub mod constant;
pub mod engine;
pub mod search;
pub mod zobrist;
