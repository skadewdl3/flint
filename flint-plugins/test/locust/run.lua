local log = require("log")
local path = require("path")
local json = require("json")
local cmd = require("cmd")
local async = require("async")

function Run(config)
    local start_server_script = path.join(path.cwd(), config.script)
    log.debug(start_server_script)

    local server_script = cmd.run_async("sh", start_server_script)

    -- local locustfile = path.join(path.cwd(), config.locustfile)

    -- local command =
    -- { "locust", "-f", locustfile, "--host", config.host, "--headless", "--users", tostring(config.users),
    --     "--spawn-rate",
    --     tostring(config.spawn_rate), "--run-time", config.run_time, "--json", "--skip-log" }

    -- local locust_cmd = cmd.run(table.unpack(command))
    -- local output = locust_cmd.output()
    -- log.debug(output)
    --
    while true do
    end



    -- cmd.kill(server_script.pid)
    return { "ls" }
end

function Eval(output)
    log.debug("evaled")
    if true then
        return {
            tests_passed = 0,
            total_tests = 0,
            passing_percentage = 0,
            test_results = {}
        }
    end
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
