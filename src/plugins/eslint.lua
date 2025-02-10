function Details()
    local details = {
        id = "eslint",
        author = "Soham Karandikar",
        version = "0.0.1",
        extensions = { "js", "ts", "mjs", "cjs" },
        category = "linter"
    }
    return details
end

function Validate(config)
    print("Validating config")
    for k, v in pairs(config) do
        print(k .. " = " .. tostring(v))
    end
    return false
end

function Generate(config)
    return ""
end
