use crate::util::plugin::download::{download_plugins, download_plugins_from_config};
use crate::util::plugin::PluginKind;
use crate::util::toml::Config;
use crate::widgets::logs::{add_log, LogKind, LogsWidget};
use clap::Parser;

use super::{AppResult, AppWidget};
use flint_macros::ui;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

#[derive(Debug)]
pub struct InstallWidget {
    logs: LogsWidget,
    args: InstallArgs,
}

#[derive(Parser, Debug)]
pub struct InstallArgs {
    /// List of plugins to install
    #[clap(short, long, value_parser, num_args = 1, value_delimiter = ' ')]
    plugins: Vec<String>,

    /// Install all linting plugins
    #[clap(short, long)]
    lint: bool,

    /// Install all testing plugins
    #[clap(short, long)]
    test: bool,

    /// Install all plugins (this is the default behaviour)
    #[clap(long, default_value_t = true)]
    all: bool,

    /// Show help for the install command
    #[clap(short, long)]
    help: bool,

    /// Show logs
    #[clap(long)]
    logs: bool,
}

impl InstallWidget {
    pub fn new(args: InstallArgs) -> Self {
        Self {
            logs: LogsWidget::default(),
            args,
        }
    }
}

impl AppWidget for InstallWidget {
    fn setup(&mut self) -> AppResult<()> {
        let toml = Config::load(std::env::current_dir().unwrap().join("flint.toml")).unwrap();
        download_plugins_from_config(&toml)?;

        let str = format!("{:#?}", self.args);
        add_log(LogKind::Debug, str);

        Ok(())
    }
}

impl WidgetRef for InstallWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        ui!((area, buf) => {
            { self.logs }
        });
    }
}
