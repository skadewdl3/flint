use crate::util::plugin::download::download_plugin;
use crate::util::plugin::{self, PluginKind};
use crate::util::toml::Config;

use super::AppWidget;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

#[derive(Debug)]
pub struct InstallWidget {}

impl Default for InstallWidget {
    fn default() -> Self {
        Self {}
    }
}

impl AppWidget for InstallWidget {}

impl WidgetRef for InstallWidget {
    fn render_ref(&self, _area: Rect, _buf: &mut Buffer) {
        let toml = Config::load(std::env::current_dir().unwrap().join("flint.toml")).unwrap();

        let linter_ids = toml.rules.keys();
        let tester_ids = toml.tests.keys();

        for id in linter_ids {
            download_plugin(PluginKind::Lint, id).unwrap()
        }

        for id in tester_ids {
            download_plugin(PluginKind::Test, id).unwrap()
        }
    }
}
