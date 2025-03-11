function Details()
    local details = {
        id = "eslint",
        author = "Onkar Kapuskari",
        version = "0.0.1",
        extensions = { "js", "ts", "mjs", "cjs" },
    }
    return details
end

function Dependencies()
    return {
        npm = {
            { name = "eslint", version = "latest" }
        },
    }
end
