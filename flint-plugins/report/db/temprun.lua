local json = require("json")
local path = require("path")
local eval = require("eval")
local log = require("log")
local sql = require("sql")
local async = require("async")


local function get_db_connection(config)
    if config.flavor == "postgres" then
        return sql.postgres({
            host = config.host,
            port = config.port,
            username = config.username,
            password = config.password,
            database = config.database
        })
    elseif config.flavor == "mysql" then
        return sql.mysql({
            host = config.host,
            port = config.port,
            username = config.username,
            password = config.password,
            database = config.database
        })
    elseif config.flavor == "sqlite" then
        log.debug(config.path)
        return sql.sqlite(
            config.path
        )
    end
end

local function ensure_table_exists(conn, table_name, create_table_sql)
    local query = string.format([[
        SELECT EXISTS (
        SELECT FROM information_schema.tables
        WHERE table_schema = 'public'
        AND table_name = '%s'
        );
    ]], table_name)

    local table_exists_result = async.block_on(sql.query, conn, query)
    local table_exists = table_exists_result[1] and table_exists_result[1].exists

    if not table_exists then
        log.info("Creating " .. table_name .. " table as it does not exist")
        local create_table_result = async.block_on(sql.execute, conn, create_table_sql)
        if not create_table_result then
            log.error("Failed to create " .. table_name .. " table")
            return false
        end
    end

    return true
end

function Run(options)
    local config = options.config
    local output = options.output
    local plugin_id = options.plugin_id

    local output_type = eval.get_output_type(output)
    output = eval.get_output(output)
    local conn = get_db_connection(config)
    local connected = async.block_on(sql.testConnection, conn)

    if not connected then
        log.error("Failed to connect to database")
        return {}
    end

    -- Ensure log table exists
    local log_table_sql = [[
        CREATE TABLE log (
            id SERIAL PRIMARY KEY,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            level TEXT,
            message TEXT,
            metadata JSONB
        );
    ]]
    ensure_table_exists(conn, "log", log_table_sql)

    -- Ensure lint_results table exists
    local lint_results_sql = [[
        CREATE TABLE lint_results (
            id SERIAL PRIMARY KEY,
            plugin_id TEXT,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            total_errors INTEGER,
            file_name TEXT,
            line_no INTEGER,
            column_no INTEGER,
            success BOOLEAN,
            error_message TEXT,
            data JSONB
        );
    ]]
    ensure_table_exists(conn, "lint_results", lint_results_sql)

    -- Ensure test_results table exists
    local test_results_sql = [[
        CREATE TABLE test_results (
            id SERIAL PRIMARY KEY,
            plugin_id TEXT,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            tests_passed INTEGER,
            total_tests INTEGER,
            passing_percentage REAL,
            file_name TEXT,
            line_no INTEGER,
            column_no INTEGER,
            success BOOLEAN,
            error_message TEXT,
            data JSONB
        );
    ]]
    ensure_table_exists(conn, "test_results", test_results_sql)

    log.info("Pushing results to database")

    -- Process results based on the output type
    if output_type == eval.lint then
        -- Process Lint results
        for _, result in ipairs(output.lint_results) do
            local data_json = result.data and json.stringify(result.data) or nil
            local insert_query = [[
                INSERT INTO lint_results
                (plugin_id, total_errors, file_name, line_no, column_no, success, error_message)
                VALUES ((1), (2), (3), (4), (5), (6), (7));
            ]]

            local insert_result = async.block_on(
                sql.execute, conn, insert_query,

                plugin_id,
                output.total_errors,
                result.file_name:gsub("'", "''"),
                result.line_no and tostring(result.line_no) or 'NULL',
                result.column_no and tostring(result.column_no) or 'NULL',
                result.success and true or false,
                result.error_message and ("'" .. tostring(result.error_message):gsub("'", "''") .. "'") or 'NULL'
            -- nil
            )
            log.warn(insert_result)
            if not insert_result then
                log.error("Failed to insert lint result for file: " .. result.file_name)
            end
        end
        log.info("Pushed " .. #output.lint_results .. " lint results to database")
    elseif output_type == eval.test then
        -- Process Test results
        for _, result in ipairs(output.test_results) do
            local data_json = result.data and json.stringify(result.data) or nil
            local insert_query = [[
                INSERT INTO test_results
                (plugin_id, tests_passed, total_tests, passing_percentage, file_name, line_no, column_no, success, error_message)
                VALUES ((1), (2), (3), (4), (5),(6),(7),(8),(9));
            ]]


            local insert_result = async.block_on(
                sql.execute, conn, insert_query,

                plugin_id,
                output.tests_passed,
                output.total_tests,
                output.passing_percentage,
                result.file_name:gsub("'", "''"),
                result.line_no and tostring(result.line_no) or nil,
                result.column_no and tostring(result.column_no) or nil,
                result.success and true or false,
                result.error_message and ("'" .. tostring(result.error_message):gsub("'", "''") .. "'") or 'NULL'
            -- nil
            )
            log.warn(insert_result)
            if not insert_result then
                log.error("Failed to insert test result for file: " .. result.file_name)
            end
        end
        log.info("Pushed " .. #output.test_results .. " test results to database")
    else
        log.error("Unknown output type")
    end

    return {}
end
