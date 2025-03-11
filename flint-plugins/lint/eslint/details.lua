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
        -- Define dependencies by type
        npm = {
            -- Format: { name = "package-name", version = "version-spec" }
            { name = "eslint", version = "^9.0.0" }
        },
    }
end
