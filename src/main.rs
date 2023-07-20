mod game;
mod ui;
fn main() -> anyhow::Result<()> {
    engine::aaa();
    let game: game::ChineseChess = Default::default();
    ui::ui(game)?;

    Ok(())
}
