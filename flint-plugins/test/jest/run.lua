function Run(config)
    local cwd = path.cwd()
    local jest_config_location = path.join(cwd, ".flint", "jest.config.js")
    return { "jest", "--config", jest_config_location, "--json" }
end

function Eval(output)
    log.debug(output)
    if not output.success then
        log.debug(output.stderr)
        return {
            tests_passed = 0,
            total_tests = 0,
            passing_percentage = 0,
            test_results = {}
        }
    end
    local output = output.stdout
    log.debug(output.stdout)
    local parsed_output = json.parse(output)
    local testResults = parsed_output.testResults

    local results = {}
    local tests_passed = 0
    local total_tests = 0

    for _, result in ipairs(testResults) do
        local file_name = result.name

        for _, assertion in ipairs(result.assertionResults) do
            total_tests = total_tests + 1

            if assertion.status == "passed" then
                tests_passed = tests_passed + 1
            end

            local test_result = {
                file_name = file_name,
                line_no = nil, -- Default values if not available
                column_no = nil,
                success = (assertion.status == "passed"),
                error_message = nil
            }

            if assertion.status == "failed" then
                -- For failed tests, extract error information if available
                if assertion.failureDetails and #assertion.failureDetails > 0 then
                    for _, failureDetail in ipairs(assertion.failureDetails) do
                        if failureDetail.line and failureDetail.column then
                            test_result.line_no = failureDetail.line
                            test_result.column_no = failureDetail.column
                        end
                    end
                end

                -- Get the error message from failureMessages if available
                if assertion.failureMessages and #assertion.failureMessages > 0 then
                    test_result.error_message = assertion.failureMessages[1]
                else
                    test_result.error_message = "Test failed without specific error message"
                end
            end

            table.insert(results, test_result)
        end
    end

    -- Create the final coverage object that includes all the information
    local coverage = {
        tests_passed = tests_passed,
        total_tests = total_tests,
        passing_percentage = total_tests > 0 and (tests_passed / total_tests * 100) or 0,
        test_results = results
    }

    -- log.debug(parsed_output)
    return coverage
end
