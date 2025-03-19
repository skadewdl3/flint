local log = require("log")
local path = require("path")
local json = require("json")
local cmd = require("cmd")
local async = require("async")
local csv = require("csv")


function Run(config)
    local locustfile = path.join(path.cwd(), config.locustfile)

    local command =
    { "locust", "-f", locustfile, "--host", config.host, "--headless", "--users", tostring(config.users or 100),
        "--spawn-rate",
        tostring(config.spawn_rate or 1), "--run-time", config.run_time or "20s", "--json", "--csv=" ..
    path.join(path.cwd(), config.output_path, "locust"),
        "--skip-log" }
    return command
end

function Eval(output, config)
    log.debug(config.output_path)
    local stats_file = path.join(path.cwd(), config.output_path, "locust_stats.csv")
    local locust_stats = csv.read(stats_file)

    log.debug(locust_stats)

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
    -- log.debug(parsed_output)

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
