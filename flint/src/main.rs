use app::{App, AppArgs};
use clap::Parser;
use flint_utils::set_flag;
use util::handle_global_flags;

pub mod app;
pub mod plugin;
pub mod util;
pub mod widgets;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let app_args = AppArgs::parse_from(&args);

    handle_global_flags(&app_args);

    // #[cfg(not(debug_assertions))]
    {
        use app::{
            help::HelpWidget, install::InstallWidget, test::TestWidget, AppWidget, AppWidgetArgs,
        };
        use threadpool::ThreadPool;
        let subcommand = args.get(1).unwrap();

        if ["test", "install"].contains(&subcommand.as_str()) {
            set_flag!(non_interactive, true);
            let mut non_interactive_widget: Box<dyn AppWidget> = match app_args.command.unwrap() {
                AppWidgetArgs::Install(args) => Box::new(InstallWidget::new(args)),
                AppWidgetArgs::Test(args) => Box::new(TestWidget::new(args)),
                _ => Box::new(HelpWidget::default()),
            };

            let thread_pool = ThreadPool::new(16);
            non_interactive_widget.set_thread_pool(&thread_pool);

            non_interactive_widget.setup().unwrap();

            thread_pool.join();
            return;
        }
    }

    let mut terminal = ratatui::init();
    let app_result = App::new(app_args).run(&mut terminal);
    app_result.expect("Error while running app");
    ratatui::restore();
}
