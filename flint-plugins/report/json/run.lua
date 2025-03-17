local json = require("json")
local path = require("path")
local eval = require("eval")
local log = require("log")

function Run(options)
    local config = options.config

    local output = options.output
    local plugin_id = options.plugin_id

    output = eval.get_output(output)


    return {
        [path.join(config.output_path, "report-" .. plugin_id .. ".json")] = json.stringify(output)
    }
end
