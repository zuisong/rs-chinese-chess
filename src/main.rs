use engine::{board, engine::UCCIEngine};

mod ui;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> anyhow::Result<()> {
    let game: board::Board = board::Board::init();

    // 加载开局库数据并创建UCCIEngine（同步加载，瞬间完成）
    let book_data = include_str!("../lib/engine/BOOK.DAT");
    let engine = UCCIEngine::new(Some(book_data));

    ui::ui(game, engine)?;
    Ok(())
}
