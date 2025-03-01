use std::path::PathBuf;

use flint_macros::ui;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;
use throbber_widgets_tui::ThrobberState;

use crate::{
    util::{
        plugin::{self, Plugin, PluginKind},
        toml::Config,
    },
    widgets::logs::LogsWidget,
};

use super::{AppResult, AppWidget};

#[derive(Debug, Default)]
pub struct TestWidget {
    logs: LogsWidget,
}

impl AppWidget for TestWidget {
    fn setup(&mut self) -> AppResult<()> {
        let plugins = plugin::list_from_config();
        let x: Vec<String> = plugins
            .iter()
            .filter_map(|plugin| match plugin.kind {
                PluginKind::Lint => None,
                PluginKind::Test => Some(plugin.details.id.clone()),
            })
            .collect();
        Ok(())
    }
}
impl WidgetRef for TestWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        ui!((area, buf) => {
            { self.logs }
        });
    }
}
