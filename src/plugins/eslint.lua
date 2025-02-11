function Details()
    local details = {
        id = "eslint",
        author = "Soham Karandikar",
        version = "0.0.1",
        extensions = { "js", "ts", "mjs", "cjs" },
        category = "linter"
    }
    return details
end

function Validate(config)
    -- sleep(2000)
    temp = { foo = 'bar', baz = 123, qux = { nested = true } }
    log.debug(temp);
    return true
end

function Generate(config)
    local common = config.common
    log.info("Generating ESLint configuration")
    log.debug(common)

    -- Determine indent rule: number for spaces, "tab" for tabs
    local indentValue = nil
    if common.indentation == "spaces" then
        indentValue = common.indent_size
    else
        indentValue = "tab"
    end

    -- Map the other settings to ESLint rule values
    local semiStyle    = common.require_semicolons and "always" or "never"
    local braceStyle   = (common.bracket_style == "kr") and "1tbs" or "allman"
    local quotesStyle  = common.quote_style -- "single" or "double"
    local trailingRule = (common.trailing_whitespace == false) and "error" or "off"

    -- Build the ESLint config file content as a string
    local result       = ""
    result             = result .. "module.exports = {\n"
    result             = result .. "  rules: {\n"

    -- indent rule: if spaces, output number; if tabs, output a quoted string
    if type(indentValue) == "number" then
        result = result .. "    indent: [\"error\", " .. indentValue .. ", { \"SwitchCase\": 1 }],\n"
    else
        result = result .. "    indent: [\"error\", \"" .. indentValue .. "\", { \"SwitchCase\": 1 }],\n"
    end

    result = result .. "    \"max-len\": [\"error\", { \"code\": " .. common.max_line_length .. " }],\n"
    result = result .. "    \"no-trailing-spaces\": \"" .. trailingRule .. "\",\n"
    result = result .. "    semi: [\"error\", \"" .. semiStyle .. "\"],\n"
    result = result .. "    \"brace-style\": [\"error\", \"" .. braceStyle .. "\"],\n"
    result = result .. "    quotes: [\"error\", \"" .. quotesStyle .. "\"]\n"

    result = result .. "  }\n"
    result = result .. "};\n"

    return {
        ["eslint.config.js"] = result
    }
end
