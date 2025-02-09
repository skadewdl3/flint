function Details()
    local details = {
        id = "eslint",
        author = "Soham Karandikar",
        version = "0.0.1",
        languages = { "javascript", "typescript" }
    }
    return details
end

function Validate(config)
    print("Validating config")
    for k, v in pairs(config) do
        print(k .. " = " .. tostring(v))
    end
    return true
end

function Generate()
    return ""
end
