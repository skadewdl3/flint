local log = require("log")
local path = require("path")
local json = require("json")

function Run(config)
    return { "chaos", "--help" }
end

function Eval(output)
    local results = {}
    local tests_passed = 0
    local total_tests = 0


    -- Create the final coverage object that includes all the information
    local coverage = {
        tests_passed = tests_passed,
        total_tests = total_tests,
        passing_percentage = 0,
        test_results = results
    }

    return coverage
end
