CONFIG_PATH=flint-tests/js/flint.toml
PLUGINS_DIR=downloaded-plugins
cargo run -- --config-path $CONFIG_PATH --plugins-dir $PLUGINS_DIR install
