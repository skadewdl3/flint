use app::App;

pub mod app;
pub mod util;
pub mod widgets;

fn main() {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    app_result.expect("Error while running app");
    ratatui::restore();
}
