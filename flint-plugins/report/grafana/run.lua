local json = require("json")
local path = require("path")
local eval = require("eval")
local log = require("log")
local sql = require("sql")


local function get_postgres_connection()
    return sql.postgres({
        host = "localhost",
        port = 5432,
        username = "postgres",
        password = "password",
        database = "mydb"
    })
end

function Run(options)
    local config = options.config
    local output = options.output
    local plugin_id = options.plugin_id

    output = eval.get_output(output)

    local conn = get_postgres_connection()
    local connected = sql.testConnection(conn)

    local res = sql.query(conn, "SELECT * FROM users")
    log.debug(res)


    return {
    }
end
