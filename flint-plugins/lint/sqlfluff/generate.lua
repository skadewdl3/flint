function Generate(config)
    config.common = nil

    local output = { tool = {} }
    output.tool.sqlfluff = config

    return {
        ["pyproject.toml"] = toml.stringify(output)
    }
end
