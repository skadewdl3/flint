function Details()
    local details = {
        id = "chaos",
        author = "Soham Karandikar",
        version = "0.0.1",
        extensions = { "*" },
    }
    return details
end

function Dependencies()
    return {
        npm = {
            { name = "chaostoolkit", version = "latest" },
        },
    }
end
