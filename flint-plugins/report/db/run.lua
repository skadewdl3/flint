local json = require("json")
local path = require("path")
local eval = require("eval")
local log = require("log")
local sql = require("sql")
local async = require("async")



function Run(options)
    local config = options.config
    local output = options.output
    local plugin_id = options.plugin_id

    local output_type = eval.get_output_type(output)
    output = eval.get_output(output)
    local conn = sql.mysql(config)
    log.info("Conencting to " .. conn)
    conn = async.block_on(sql.connect, conn)
    log.success("Connected to database")
    -- log.debug(conn)
    local res = conn:execute_sync("INSERT INTO users (name, email, age) VALUES ('John Doe', 'john@doe.com', 30)")
    log.debug(res)

    res = conn:query_sync("SELECT * FROM users")
    log.debug(res)

    -- Ensure log table exists

    return {}
end
