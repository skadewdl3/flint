local log = require("log")
local path = require("path")
local json = require("json")

function Run(config)
    local cwd = path.cwd()
    local files = path.ls(cwd)
    -- return { "jest", "--json", "--passWithNoTests", "--logHeapUsage", "--testLocationInResults" }
    return { "chaos", "run" }
end

function Eval(output)
    -- if not output.success then
    --     return {
    --         tests_passed = 0,
    --         total_tests = 0,
    --         passing_percentage = 0,
    --         test_results = {}
    --     }
    -- end
    -- local output = output.stdout
    -- local parsed_output = json.parse(output)
    -- local testResults = parsed_output.testResults

    local results = {}
    local tests_passed = 0
    local total_tests = 0

    -- Create the final coverage object that includes all the information
    local coverage = {
        tests_passed = tests_passed,
        total_tests = total_tests,
        passing_percentage = total_tests > 0 and (tests_passed / total_tests * 100) or 0,
        test_results = results
    }

    return coverage
end
