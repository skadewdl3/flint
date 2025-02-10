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
    print("Validating config")
    print(config)
    return true
end

function Generate()
    return ""
end
