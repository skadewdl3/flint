local json = require("json")
local path = require("path")
local eval = require("eval")
local log = require("log")
local sql = require("sql")
local async = require("async")


local function ensure_table_exists(conn, table_name, create_table_sql)
    local query = string.format([[
        SELECT COUNT(*) as table_exists
        FROM information_schema.tables
        WHERE table_schema = DATABASE()
        AND table_name = '%s';
        ]], table_name)

    local table_exists_result = async.await(conn.query, conn, query)
    local table_exists = table_exists_result[1] and table_exists_result[1].table_exists

    if not table_exists then
        log.info("Creating " .. table_name .. " table as it does not exist")
        local create_table_result = async.await(conn.execute, conn, create_table_sql)
        if not create_table_result then
            log.error("Failed to create " .. table_name .. " table")
            return false
        end
    end

    return true
end

-- Gets database connection details from environment variables
local function get_db_config(config)
    local env = require("env")
    local output = {}
    local keys = { "host", "port", "username", "password", "database" }
    for _, key in ipairs(keys) do
        log.debug(config[key])
        output[key] = env.var(env.var_name(config[key]))
    end
    return output
end

function Run(options)
    local config = options.config

    local output = options.output
    local plugin_id = options.plugin_id

    local output_type = eval.get_output_type(output)
    output = eval.get_output(output)
    local conn = sql.mysql(get_db_config(config))
    conn = async.await(sql.connect, conn)

    -- Test connection
    local connected = conn ~= nil
    if not connected then
        log.error("Failed to connect to database")
        return {}
    end

    -- Ensure lint_results table exists
    local lint_results_sql = [[
    CREATE TABLE lint_results (
    id INT AUTO_INCREMENT PRIMARY KEY,
    plugin_id VARCHAR(255),
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    total_errors INT,
    file_name VARCHAR(255),
    line_no INT,
    column_no INT,
    success BOOLEAN,
    error_message TEXT,
    data JSON
    ) ENGINE=InnoDB;
    ]]
    ensure_table_exists(conn, "lint_results", lint_results_sql)

    -- Ensure test_results table exists
    local test_results_sql = [[
    CREATE TABLE test_results (
    id INT AUTO_INCREMENT PRIMARY KEY,
    plugin_id VARCHAR(255),
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    tests_passed INT,
    total_tests INT,
    passing_percentage FLOAT,
    file_name VARCHAR(255),
    line_no INT,
    column_no INT,
    success BOOLEAN,
    error_message TEXT,
    data JSON
    ) ENGINE=InnoDB;
    ]]
    ensure_table_exists(conn, "test_results", test_results_sql)

    log.info("Pushing results to database")

    -- Process results based on the output type
    if output_type == eval.lint then
        -- Process Lint results
        for _, result in ipairs(output.lint_results) do
            -- Using parameterized queries with the new SQL interface
            local insert_query = [[
            INSERT INTO lint_results
            (plugin_id, total_errors, file_name, line_no, column_no, success, error_message)
            VALUES (?, ?, ?, ?, ?, ?, ?);
            ]]

            local insert_result = async.await(
                conn.execute,
                conn,
                insert_query,
                sql.string(plugin_id),
                sql.int(output.total_errors),
                sql.string(result.file_name),
                sql.int(result.line_no or nil),
                sql.int(result.column_no or nil),
                sql.bool(result.success and true or false),
                sql.string(result.error_message or nil)
            )

            if not insert_result then
                log.error("Failed to insert lint result for file: " .. result.file_name)
            end
        end
        log.success("Pushed " .. #output.lint_results .. " lint results to database")
    elseif output_type == eval.test then
        -- Process Test results
        for _, result in ipairs(output.test_results) do
            local insert_query = [[
            INSERT INTO test_results
            (plugin_id, tests_passed, total_tests, passing_percentage, file_name, line_no, column_no, success, error_message)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
            ]]

            local insert_result = async.await(
                conn.execute,
                conn,
                insert_query,
                sql.string(plugin_id),
                sql.int(output.tests_passed),
                sql.int(output.total_tests),
                sql.float(output.passing_percentage),
                sql.string(result.file_name),
                sql.int(result.line_no or nil),
                sql.int(result.column_no or nil),
                sql.bool(result.success and true or false),
                sql.string(result.error_message or nil)
            )

            if not insert_result then
                log.error("Failed to insert test result for file: " .. result.file_name)
            end
        end
        log.success("Pushed " .. #output.test_results .. " test results to database")
    else
        log.error("Unknown output type")
    end

    return {}
end
