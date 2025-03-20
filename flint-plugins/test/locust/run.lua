local log = require("log")
local path = require("path")
local json = require("json")
local cmd = require("cmd")
local async = require("async")
local csv = require("csv")
local json = require("json")
local sql = require("sql")
local eval = require("eval")


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

local function ensure_table_exists(conn, table_name, create_table_sql)
    local query = string.format([[SELECT COUNT(*) as table_exists FROM information_schema.tables
        WHERE table_schema = DATABASE() AND table_name = '%s';]], table_name)

    local table_exists_result = async.await(conn.query, conn, query)
    local table_exists = table_exists_result[1] and table_exists_result[1].table_exists == 1

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

function Eval(output, options)
    local conn = sql.mysql(get_db_config(options.env))
    conn = async.await(sql.connect, conn)
    if not conn then
        log.error("Failed to connect to database")
        return {}
    end

    local performance_results_sql = [[
        CREATE TABLE performance_results (
            id INT AUTO_INCREMENT PRIMARY KEY,
            plugin_id VARCHAR(255),
            timestamp BIGINT,
            name VARCHAR(255),
            request_count INT,
            failure_count INT,
            requests_per_second FLOAT,
            median_response_time FLOAT,
            average_response_time FLOAT,
            min_response_time FLOAT,
            max_response_time FLOAT,
            percentile_50 FLOAT,
            percentile_75 FLOAT,
            percentile_90 FLOAT,
            percentile_95 FLOAT,
            percentile_99 FLOAT,
            percentile_99_9 FLOAT,
            percentile_100 FLOAT,
            average_content_size FLOAT
        ) ENGINE=InnoDB;
        ]]
    ensure_table_exists(conn, "performance_results", performance_results_sql)



    local insert_query = [[
        INSERT INTO performance_results
        (plugin_id, timestamp, name, request_count, failure_count, requests_per_second,
        median_response_time, average_response_time, min_response_time, max_response_time,
        percentile_50, percentile_75, percentile_90, percentile_95, percentile_99,
        percentile_99_9, percentile_100, average_content_size)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        ]]


    local csv_data = csv.read(path.join(path.cwd(), options.output_path, "locust_stats_history.csv"))

    for _, result in ipairs(csv_data) do
        log.debug(result)
        local insert_result = async.await(
            conn.execute,
            conn,
            insert_query,
            sql.string("locust"),
            sql.big_int(tonumber(result["Timestamp"]) or 0),
            sql.string(result["Name"] or "Unknown"),
            sql.int(tonumber(result["Request Count"]) or 0),
            sql.int(tonumber(result["Failure Count"]) or 0),
            sql.float(tonumber(result["Requests/s"]) or 0.0),
            sql.float(tonumber(result["Median Response Time"]) or 0.0),
            sql.float(tonumber(result["Average Response Time"]) or 0.0),
            sql.float(tonumber(result["Min Response Time"]) or 0.0),
            sql.float(tonumber(result["Max Response Time"]) or 0.0),
            sql.float(tonumber(result["50%"]) or 0.0),
            sql.float(tonumber(result["75%"]) or 0.0),
            sql.float(tonumber(result["90%"]) or 0.0),
            sql.float(tonumber(result["95%"]) or 0.0),
            sql.float(tonumber(result["99%"]) or 0.0),
            sql.float(tonumber(result["99.9%"]) or 0.0),
            sql.float(tonumber(result["100%"]) or 0.0),
            sql.float(tonumber(result["Average Content Size"]) or 0.0)
        )


        if not insert_result then
            log.error("Failed to insert Locust performance result")
        else
            log.success("Pushed Locust results to database")
        end
    end


    local results = {}
    local tests_passed = 0
    local total_tests = 0
    local coverage = {
        tests_passed = tests_passed,
        total_tests = total_tests,
        passing_percentage = total_tests > 0 and (tests_passed / total_tests * 100) or 0,
        test_results = results
    }

    return coverage
end
