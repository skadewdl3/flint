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
            { name = "@eslint/js",                   version = "latest" },
            { name = "@eslint/js",                   version = "latest" },
            { name = "@stylistic/eslint-plugin",     version = "latest" },
            { name = "@stylistic/eslint-plugin-js",  version = "latest" },
            { name = "@stylistic/eslint-plugin-jsx", version = "latest" },
            { name = "@stylistic/eslint-config",     version = "latest" },
        },
    }
end
