use engine::board;

mod ui;

fn main() -> anyhow::Result<()> {
    let game: board::Board = board::Board::init();
    ui::ui(game)?;
    Ok(())
}
