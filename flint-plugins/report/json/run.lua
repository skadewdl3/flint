function Run(options)
    local config = options.config
    local output = options.output


    -- Generate a random ID
    local id = math.random(100000, 999999)

    return {
        [config.outputPath .. "/report-" .. id .. ".json"] = json.stringify(output)
    }
end
