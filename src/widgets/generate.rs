use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, Mutex},
};

use mlua::{Function, Lua};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Text,
    Frame,
};

use super::{AppStatus, AppWidget};
use crate::util::{
    get_plugin_map,
    plugin::{run_plugin, Plugin},
    thread_manager::ThreadManager,
    toml::read_toml_config,
};

pub struct GenerateWidget {
    plugins: Vec<Plugin>,
    thread_manager: Option<ThreadManager>,
    lua: Lua,
    logs: Arc<Mutex<Vec<String>>>,
}

impl Default for GenerateWidget {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            thread_manager: None,
            lua: Lua::new(),
            logs: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl AppWidget for GenerateWidget {
    fn register_thread_manager(&mut self, thread_manager: ThreadManager) {
        self.thread_manager = Some(thread_manager)
    }

    fn setup(&mut self) -> AppStatus {
        let toml = Arc::new(read_toml_config("./flint.toml").unwrap());
        let plugin_ids = toml.linters.keys().collect::<Vec<&String>>();

        let globals = self.lua.globals();

        let logs = self.logs.clone();
        let log = self
            .lua
            .create_function(move |_, message: String| {
                let _ = logs;
                logs.lock().unwrap().push(message);
                Ok(())
            })
            .unwrap();
        globals.set("log", log).unwrap();

        self.plugins = get_plugin_map()
            .values()
            .flat_map(|plugin_set| plugin_set.iter())
            .collect::<BTreeSet<&Plugin>>()
            .into_iter()
            .filter(|plugin| plugin_ids.contains(&&plugin.details.id))
            .cloned()
            .collect();

        for plugin in &self.plugins {
            println!("Running plugin: {}", plugin.details.id);
            let plugin_clone = plugin.clone();
            let toml_clone = toml.clone();
            let lua = self.lua.clone();
            let handle = std::thread::spawn(move || run_plugin(lua, toml_clone, &plugin_clone));
            if let Some(thread_manager) = &self.thread_manager {
                thread_manager.add_thread(handle);
            }
        }

        AppStatus::Ok
    }

    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        if let Ok(logs) = self.logs.lock() {
            let length = logs.len();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(Constraint::from_lengths(vec![1; length]))
                .split(frame.area());

            for i in 0..logs.len() {
                frame.render_widget(Text::raw(logs[i].clone()), layout[i]);
            }
        }

        AppStatus::Ok
    }
}
