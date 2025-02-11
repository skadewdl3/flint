function Details()
    local details = {
        id = "sqlfluff",
        author = "Soham Karandikar",
        version = "0.0.1",
        extensions = { "sql" },
        category = "linter"
    }
    return details
end

function Validate(config)
    log("Validating sqlfluff config")
    return true
end

function Generate()
    log("Generating sqlfluff linter file")
    return {
    }
end
