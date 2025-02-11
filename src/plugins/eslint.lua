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
    log.info("Generating eslint linter file")
    local output = "module.exports = {\n"

    if config.semi ~= nil then
        output = output .. string.format("    semi: %s,\n", tostring(config.semi))
    end

    output = output .. "};\n"

    return {
        ["eslint.config.js"] = output
    }
end
