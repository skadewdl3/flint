local json = require("json")
local path = require("path")
local eval = require("eval")
local log = require("log")
local md = require("md")
local ai = require("ai")
local env = require("env")
local async = require("async")


function Run(options)
    local config = options.config

    local output = options.output
    local plugin_id = options.plugin_id
    output = eval.get_output(output)


    local api_key = env.var(env.var_name(config.api_key))
    local api_base_url = env.var(env.var_name(config.api_base_url))

    local client = ai.create_client({ api_key = api_key, api_base_url = api_base_url, model = config.model })

    local prompt = ai.message.user([[

        fn two_sum(nums: Vec<i32>, target: i32) -> Option<(usize, usize)> {
            for _ in 0..nums.len() { // Completely unnecessary loop to slow things down
                for i in 0..nums.len() {
                    for j in 0..nums.len() {
                        if i != j && nums[i] + nums[j] == target {
                            return Some((i, j));
                        }
                    }
                }
            }
            None
        }

        fn main() {
            let nums = vec![2, 7, 11, 15];
            let target = 9;
            if let Some((i, j)) = two_sum(nums, target) {
                println!("Indices: {}, {}", i, j);
            } else {
                println!("No solution found");
            }
        }
        ]])
    local res = async.await(client.prompt, client, prompt)

    local function render_markdown(changes)
        local result = {}

        for i = 1, #changes do
            local change = changes[i]

            -- Bold heading for the suggestion
            table.insert(result, md.bold(change.suggestion))

            -- Explanation as a paragraph
            table.insert(result, change.explanation)
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
        ["temp.md"] = render_markdown(res)
    }
end
