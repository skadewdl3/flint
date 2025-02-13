function Details()
    local details = {
        id = "sqlfluff",
        author = "Soham Karandikar",
        version = "0.0.1",
        extensions = { "sql" },
        category = "lint"
    }
    return details
end

function Validate(config)
    log.info("Validating sqlfluff config")
    return true
end

function Generate()
    log.info("Generating sqlfluff linter file")
    return {
    }
end
