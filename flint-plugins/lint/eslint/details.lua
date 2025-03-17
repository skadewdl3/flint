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
            { name = "eslint",                      version = "latest" },
            { name = "@stylisic/eslint-plugin",     version = "latest" },
            { name = "@stylisic/eslint-plugin-js",  version = "latest" },
            { name = "@stylisic/eslint-plugin-jsx", version = "latest" },
            { name = "@stylisic/eslint-config",     version = "latest" },
        },
    }
end
