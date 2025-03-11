function Details()
    local details = {
        id = "jest",
        author = "Aditya Sakhare",
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
            { name = "jest",   version = "^29.0.0" },
            { name = "eslint", version = "^9.0.0" }
        },
        pip = {
            { name = "pytest", version = ">=7.0.0" }
        },
    }
end
