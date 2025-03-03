function Run(config)
    return { "eslint", "--help" }
end

function Eval(output)
    log.debug(output)
    local output = output.stdout;
    local parsed_output = json.parse(output)
    log.debug(parsed_output)
    return true
end
