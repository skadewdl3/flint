function Details()
    local details = {
        id = "locust",
        author = "Soham Karandikar",
        version = "0.0.1",
        extensions = { "*" },
    }
    return details
end

function Dependencies()
    return {
        pip = {
            { name = "locust", version = "latest" },
        },
    }
end
