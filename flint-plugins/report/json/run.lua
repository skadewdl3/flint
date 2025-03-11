local json = require("json")

function Run(options)
    local config = options.config
    local output = options.output

    return {
        [config.outputPath .. "/report.json"] = json.stringify(output)
    }
end
