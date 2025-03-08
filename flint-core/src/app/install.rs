use crate::util::plugin::download::download_plugins;
use crate::util::plugin::PluginKind;
use crate::util::toml::Config;
use crate::widgets::logs::LogsWidget;

use super::{AppResult, AppWidget};
use flint_macros::ui;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

#[derive(Debug)]
pub struct InstallWidget {
    logs: LogsWidget,
}

impl Default for InstallWidget {
    fn default() -> Self {
        Self {
            logs: LogsWidget::default(),
        }
    }
}

impl AppWidget for InstallWidget {
    fn setup(&mut self) -> AppResult<()> {
        let toml = Config::load(std::env::current_dir().unwrap().join("flint.toml")).unwrap();

        let linter_ids: Vec<&String> = toml.rules.keys().collect();
        let tester_ids: Vec<&String> = toml.tests.keys().collect();

        download_plugins(PluginKind::Test, tester_ids);
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
