local log = require("log")
local path = require("path")
local json = require("json")

function Run(config)
    local cwd = path.cwd()
    local files = path.ls(cwd)
    local locustfile = path.join(path.cwd(), config.locustfile)
    local command =
    { "locust", "-f", locustfile, "--host", config.host, "--headless", "--users", tostring(config.users),
        "--spawn-rate",
        tostring(config.spawn_rate), "--run-time", config.run_time, "--json", "--skip-log" }
    log.debug(table.concat(command, " "))
    return command
end

function Eval(output)
    log.debug(output)
    if not output.success then
        return {
            tests_passed = 0,
            total_tests = 0,
            passing_percentage = 0,
            test_results = {}
        }
    end
    output = output.stdout
    local parsed_output = json.parse(output)
    log.debug(parsed_output)

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
