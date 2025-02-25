function Generate(config)
    config.common = nil

    local output = { tool = {} }
    output.tool.sqlfluff = config

    return {
        ["pyproject.toml"] = to_toml(output)
    }
end
