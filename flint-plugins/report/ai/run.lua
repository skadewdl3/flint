local json = require("json")
local path = require("path")
local eval = require("eval")
local log = require("log")
local md = require("md")
local ai = require("ai")
local env = require("env")
local async = require("async")
local fs = require("fs")

local function group_test_results(test_results)
    local grouped = {}

    for _, result in ipairs(test_results) do
        local file = result.file_name
        if not grouped[file] then
            grouped[file] = { file_name = file, issues = {} }
        end
        table.insert(grouped[file].issues, {
            error_message = result.error_message,
            line_no = result.line_no,
            column_no = result.column_no,
            success = result.success,
            title = result.data.title
        })
    end

    -- Convert the grouped table to an array
    local output = {}
    for _, v in pairs(grouped) do
        table.insert(output, v)
    end

    return output
end

function Run(options)
    local config = options.config

    local output = options.output
    local plugin_id = options.plugin_id
    local type = eval.get_output_type(output)
    output = eval.get_output(output)


    local api_key = env.var(env.var_name(config.env.API_KEY))
    local api_base_url = env.var(env.var_name(config.env.API_BASE_URL))
    local client = ai.create_client({ api_key = api_key, api_base_url = api_base_url, model = config.env.MODEL })


    if type == eval.lint then
        return {}
    end

    local test_results = output.test_results
    local ai_input = group_test_results(test_results)
    local bruh = fs.get_ai_input(path.cwd())

    local prompt = string.format([[
        Here is my file structure and contents:
        %s

        Here are my error messages:
        %s

    ]], bruh, json.stringify(ai_input))
    prompt = ai.message.user(prompt)

    log.debug("Prompting ai")
    local res = async.await(client.prompt, client, prompt)
    local repo_branch_url = env.var_unsafe("REPO_BRANCH_URL")

    local function render_markdown(changes)
        local result = {}

        for i = 1, #changes do
            local change = changes[i]

            -- Bold heading for the suggestion
            table.insert(result, md.bold(change.suggestion))

            -- Explanation as a paragraph
            table.insert(result, change.explanation)
            table.insert(result,
                md.text(
                    (repo_branch_url or "") ..
                    change.file,
                    "/",
                    change.line_no
                )
            )
            table.insert(result, md.newline())

            -- Code block with syntax highlighting
            if change.code_changes and change.code_changes ~= "" then
                table.insert(result, md.code(change.code_language, change.code_changes))
            end

            -- Add a newline for spacing
            table.insert(result, md.newline())
        end

        return table.concat(result, "\n")
    end

    return {
        [path.join(path.cwd(), config.output_path, "temp.md")] = render_markdown(res)
    }
end
