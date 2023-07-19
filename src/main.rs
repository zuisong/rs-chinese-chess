mod game;
mod ui;
fn main() -> anyhow::Result<()> {
    let game: game::ChineseChess = Default::default();
    ui::ui(game)?;
    Ok(())
}
