local log = require("log")
local json = require("json")
local path = require("path")

function Run(config)
    local extra = config.config
    local args = { "npx", "eslint", "--format", "json" }

    return args
end

function Eval(output)
    output = output.stdout
    log.warn(output)
    local parsed_output = json.parse(output)
    -- log.debug(parsed_output)
    local cwd = path.cwd()

    -- Extract linting results
    local results = {}

    -- Check if parsed_output is in the expected format (array of file results)
    if type(parsed_output) == "table" then
        for _, file_result in ipairs(parsed_output) do
            local file_path = file_result.filePath

            -- Process messages (errors and warnings)
            if file_result.messages and #file_result.messages > 0 then
                for _, msg in ipairs(file_result.messages) do
                    local result = {
                        file_name = path.relative(file_path, cwd),
                        -- rule_id = msg.ruleId,
                        line_no = msg.line,
                        column_no = msg.column,
                        error_message = msg.message,
                        success = false,
                        data = {
                            severity = msg.severity,
                            rule_id = msg.ruleId,
                            node_type = msg.nodeType,
                            message_id = msg.messageId
                        }
                    }

                    table.insert(results, result)
                end
            end
        end
    end

    -- log.debug(results)

    -- Return false if there are linting errors, true otherwise
    return {
        total_errors = #results,
        lint_results = results
    }
end
