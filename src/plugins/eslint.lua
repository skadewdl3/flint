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
    local indentValue = (common.indentation == "spaces") and common.indent_size or "tab"

    -- Prepare the ESLint config as a Lua table
    local eslintConfig = {
        {
            rules = {
                indent = { "error", indentValue, { SwitchCase = 1 } },
                ["max-len"] = { "error", { code = common.max_line_length } },
                ["no-trailing-spaces"] = (common.trailing_whitespace == false) and "error" or "off",
                semi = { "error", common.require_semicolons and "always" or "never" },
                ["brace-style"] = { "error", (common.bracket_style == "kr") and "1tbs" or "allman" },
                quotes = { "error", common.quote_style }
            }
        }
    }

    -- Convert the table to a JSON string
    return {
        ["eslint.config.js"] = "export default " .. to_json(eslintConfig) .. ";"
    }
end
