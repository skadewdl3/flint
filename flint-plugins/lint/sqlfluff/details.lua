function Details()
    local details = {
        id = "sqlfluff",
        author = "Onkar Kapuskari",
        version = "0.0.1",
        extensions = { "sql" },
    }
    return details
end

function Generate()
    log.info("Generating sqlfluff linter file")
    return {
    }
end

function Dependencies()
    return {
        pip = {
            { name = "sqlfluff", version = "latest" }
        },
    }
end
