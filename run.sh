CONFIG_PATH=flint-tests/locust/flint.toml
PLUGINS_DIR=flint-plugins
cargo run -- --no-install --config-path $CONFIG_PATH --plugins-dir $PLUGINS_DIR test
