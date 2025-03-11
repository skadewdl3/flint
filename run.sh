CONFIG_PATH=flint-tests/js/flint.toml
PLUGINS_DIR=flint-plugins
target/debug/flint --no-install --config-path $CONFIG_PATH --plugins-dir $PLUGINS_DIR  generate
