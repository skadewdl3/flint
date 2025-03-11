function Details()
    local details = {
        id = "jest",
        author = "Aditya Sakhare (test)",
        version = "0.0.1",
        extensions = { "js", "ts", "mjs", "cjs" },
    }
    return details
end

function Dependencies()
    return {
        npm = {
            { name = "jest", version = "latest" },
        },
    }
end
