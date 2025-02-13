pub mod util;
pub mod widgets;

fn main() {
    let mut terminal = ratatui::init();
    let app_result = widgets::App::new().run(&mut terminal);
    app_result.expect("Error while running app");
    ratatui::restore();
}
