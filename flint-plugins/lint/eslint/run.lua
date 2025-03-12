local log = require("log")
local json = require("json")
local path = require("path")

function Run(config)
    local extra = config.config

    -- local args = { "npx", "eslint", "--format", "json", "src" } -- ideal output
    local args = { "npx", "eslint", "--format", "json" }

    return args
end

function Eval(output)
    local json_output = json.parse(output.stdout)
    log.debug(json_output)



    local results = {}
    local tests_passed = 0
    local total_tests = 0
    local coverage = {
        tests_passed = tests_passed,
        total_tests = total_tests,
        passing_percentage = 0,
        test_results = results
    }

    return coverage
end
