local json = require("json")
local path = require("path")

function Run(options)
    local config = options.config
    local output = options.output

    return {
        [path.join(config.output_path, "report.json")] = json.stringify(output)
    }
end
