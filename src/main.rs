use engine::{board, engine::UCCIEngine};

mod ui;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> anyhow::Result<()> {
    let game: board::Board = board::Board::init();
    let engine = UCCIEngine::new(Some(engine::book_data()));

    ui::ui(game, engine)?;
    Ok(())
}
