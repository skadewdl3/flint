local log = require("log")
local json = require("json")

function Run(config)
    return { "npx", "eslint", "--yes" }
end

function Eval(output)
    log.debug(output)
    local output = output.stdout;
    local parsed_output = json.parse(output)
    log.debug(parsed_output)
    return true
end
