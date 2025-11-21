/**
 * 详细中文注释 - 引擎核心库入口
 *
 * 说明
 * - 暴露三个子模块：board, constant, engine, zobrist
 * - 这些模块共同构成象棋引擎的核心逻辑：棋盘状态、棋子类型、哈希、以及对外的引擎实现
 */
pub mod board;
pub mod constant;
pub mod engine;
pub mod zobrist;
