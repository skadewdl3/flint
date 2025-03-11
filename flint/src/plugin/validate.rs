use crate::{
    app::AppResult,
    app_err,
    plugin::{Plugin, PluginKind},
};

pub fn validate_plugin_structure(plugin: &Plugin) -> AppResult<()> {
    let required_files = match plugin.kind {
        PluginKind::Lint => vec!["details.lua", "generate.lua", "run.lua", "validate.lua"],
        PluginKind::Test => vec!["details.lua", "generate.lua", "run.lua", "validate.lua"],
        PluginKind::Ci => vec!["details.lua", "generate.lua", "validate.lua"], // No run.lua needed
        PluginKind::Report => vec!["details.lua", "run.lua", "validate.lua"], // No generate.lua needed
    };

    for file in required_files {
        let file_path = plugin.path.join(file);
        if !file_path.exists() {
            return Err(app_err!(
                "Plugin {} is missing required file: {}",
                plugin.details.id,
                file
            ));
        }
    }

    Ok(())
}
