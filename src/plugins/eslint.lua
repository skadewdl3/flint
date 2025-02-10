
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
    sleep(2000)
    log("Validating eslint config")
    return true
end

function Generate(config)
    log("Generating eslint linter file")
    return ""
end
