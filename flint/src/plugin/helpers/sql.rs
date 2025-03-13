use sqlx::Column;
use sqlx::Executor;
use std::sync::{Arc, LazyLock, Mutex};

use mlua::{Lua, Result as LuaResult, Table, Value};
use sqlx::AnyPool;
use sqlx::{
    any::{AnyKind, AnyPoolOptions},
    pool::PoolConnection,
    query::Query,
    Row,
};

use crate::{app::AppResult, debug, error};

// Cache for database connections

static DB_POOLS: LazyLock<Arc<Mutex<std::collections::HashMap<String, AnyPool>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(std::collections::HashMap::new())));

// Helper function to build connection string from parameters
fn build_connection_string(params: &Table) -> Result<String, String> {
    // Get database type (required)
    let db_type: String = match params.get("type") {
        Ok(db_type) => db_type,
        Err(_) => return Err("Database type is required".to_string()),
    };

    match db_type.as_str() {
        "postgres" => {
            let host: String = params
                .get("host")
                .unwrap_or_else(|_| "localhost".to_string());
            let port: u16 = params.get("port").unwrap_or(5432);
            let username: String = params.get("username").unwrap_or_default();
            let password: String = params.get("password").unwrap_or_default();
            let database: String = params.get("database").unwrap_or_default();

            Ok(format!(
                "postgres://{}:{}@{}:{}/{}",
                username, password, host, port, database
            ))
        }
        "mysql" => {
            let host: String = params
                .get("host")
                .unwrap_or_else(|_| "localhost".to_string());
            let port: u16 = params.get("port").unwrap_or(3306);
            let username: String = params.get("username").unwrap_or_default();
            let password: String = params.get("password").unwrap_or_default();
            let database: String = params.get("database").unwrap_or_default();

            Ok(format!(
                "mysql://{}:{}@{}:{}/{}",
                username, password, host, port, database
            ))
        }
        "sqlite" => {
            let path: String = match params.get("path") {
                Ok(path) => path,
                Err(_) => return Err("SQLite requires a path parameter".to_string()),
            };

            Ok(format!("sqlite://{}", path))
        }
        _ => Err(format!("Unsupported database type: {}", db_type)),
    }
}

// Helper function to create a connection key for the connection pool cache
fn create_connection_key(params: &Table) -> Result<String, String> {
    // Get database type (required)
    let db_type: String = match params.get("type") {
        Ok(db_type) => db_type,
        Err(_) => return Err("Database type is required".to_string()),
    };

    match db_type.as_str() {
        "postgres" | "mysql" => {
            let host: String = params
                .get("host")
                .unwrap_or_else(|_| "localhost".to_string());
            let port: u16 =
                params.get("port").unwrap_or_else(
                    |_| {
                        if db_type == "postgres" {
                            5432
                        } else {
                            3306
                        }
                    },
                );
            let username: String = params.get("username").unwrap_or_default();
            let database: String = params.get("database").unwrap_or_default();

            Ok(format!(
                "{}:{}@{}:{}/{}",
                db_type, username, host, port, database
            ))
        }
        "sqlite" => {
            let path: String = match params.get("path") {
                Ok(path) => path,
                Err(_) => return Err("SQLite requires a path parameter".to_string()),
            };

            Ok(format!("sqlite:{}", path))
        }
        _ => Err(format!("Unsupported database type: {}", db_type)),
    }
}

async fn get_db_pool(params: &Table) -> Result<AnyPool, String> {
    // Create a unique key for the connection
    let conn_key = create_connection_key(params)?;

    // First, check if we already have a pool for this connection
    let existing_pool = {
        let pools = DB_POOLS.lock().unwrap();
        pools.get(&conn_key).cloned()
    };

    if let Some(pool) = existing_pool {
        return Ok(pool);
    }

    // If we don't have a pool, create a new one
    let conn_str = build_connection_string(params)?;
    debug!(
        "Connecting to database with connection string: {}",
        conn_str
    );

    let pool = match AnyPoolOptions::new()
        .max_connections(5)
        .connect(&conn_str)
        .await
    {
        Ok(pool) => pool,
        Err(err) => return Err(format!("Failed to connect to database: {}", err)),
    };

    // Store the pool for reuse, but only after creating it
    {
        let mut pools = DB_POOLS.lock().unwrap();
        // Check again if someone else created a pool while we were creating ours
        if let Some(existing_pool) = pools.get(&conn_key) {
            // Someone else created a pool, use that instead
            return Ok(existing_pool.clone());
        }
        pools.insert(conn_key.clone(), pool.clone());
    }

    Ok(pool)
}

// Helper function to convert a SQLx row to a Lua table
fn row_to_lua_table(lua: &Lua, row: sqlx::any::AnyRow) -> LuaResult<Table> {
    let table = lua.create_table()?;

    for (i, column) in row.columns().iter().enumerate() {
        let column_name = column.name();

        // Try to get value based on type
        if let Ok(val) = row.try_get::<i64, _>(i) {
            table.set(column_name, val)?;
        } else if let Ok(val) = row.try_get::<f64, _>(i) {
            table.set(column_name, val)?;
        } else if let Ok(val) = row.try_get::<String, _>(i) {
            table.set(column_name, val)?;
        } else if let Ok(val) = row.try_get::<bool, _>(i) {
            table.set(column_name, val)?;
        } else if let Ok(val) = row.try_get::<Vec<u8>, _>(i) {
            // For binary data, convert to a Lua string
            let lua_str = lua.create_string(&val)?;
            table.set(column_name, lua_str)?;
        } else {
            // If we can't determine the type, try to get it as a string
            match row.try_get::<Option<String>, _>(i) {
                Ok(Some(val)) => table.set(column_name, val)?,
                Ok(None) => table.set(column_name, Value::Nil)?,
                Err(_) => table.set(column_name, Value::Nil)?, // Default to nil for unknown types
            }
        }
    }

    Ok(table)
}

// Helper function to execute a query and convert results to Lua tables
async fn execute_query(
    lua: &Lua,
    conn_params: &Table,
    query_str: &str,
    params: Vec<Value>,
) -> LuaResult<Table> {
    // Get database pool
    let pool = match get_db_pool(conn_params).await {
        Ok(pool) => pool,
        Err(err) => return Err(mlua::Error::external(err)),
    };

    // Create the results table
    let results = lua.create_table()?;

    // Build the query with parameters
    let mut query = sqlx::query(query_str);

    // Add parameters
    for param in params {
        query = match param {
            Value::String(s) => query.bind(s.to_string_lossy()),
            Value::Integer(i) => query.bind(i),
            Value::Number(n) => query.bind(n),
            Value::Boolean(b) => query.bind(b),
            Value::Nil => query.bind(Option::<String>::None),
            _ => {
                return Err(mlua::Error::external(format!(
                    "Unsupported parameter type: {:?}",
                    param
                )))
            }
        };
    }

    // Execute the query
    match query.fetch_all(&pool).await {
        Ok(rows) => {
            for (i, row) in rows.iter().enumerate() {
                let lua_row = row_to_lua_table(lua, row.clone())?;
                results.set(i + 1, lua_row)?;
            }
        }
        Err(err) => {
            return Err(mlua::Error::external(format!("Query error: {}", err)));
        }
    }

    Ok(results)
}

// Helper function to execute a query that doesn't return rows
async fn execute_statement(
    conn_params: &Table,
    query_str: &str,
    params: Vec<Value>,
) -> LuaResult<i64> {
    // Get database pool
    let pool = match get_db_pool(conn_params).await {
        Ok(pool) => pool,
        Err(err) => return Err(mlua::Error::external(err)),
    };

    // Build the query with parameters
    let mut query = sqlx::query(query_str);

    // Add parameters
    for param in params {
        query = match param {
            Value::String(s) => query.bind(s.to_string_lossy().to_string()),
            Value::Integer(i) => query.bind(i),
            Value::Number(n) => query.bind(n),
            Value::Boolean(b) => query.bind(b),
            Value::Nil => query.bind(Option::<String>::None),
            _ => {
                return Err(mlua::Error::external(format!(
                    "Unsupported parameter type: {:?}",
                    param
                )))
            }
        };
    }

    // Execute the query
    match query.execute(&pool).await {
        Ok(result) => Ok(result.rows_affected() as i64),
        Err(err) => Err(mlua::Error::external(format!("Statement error: {}", err))),
    }
}

pub fn sql_helpers(lua: &Lua) -> AppResult<Table> {
    let sql = lua.create_table()?;

    // Function to query database and return results as table
    let query_fn = lua.create_function(|lua, args: mlua::Variadic<Value>| {
        if args.len() < 2 {
            return Err(mlua::Error::external(
                "sql.query requires at least 2 arguments: connection_params and query_string",
            ));
        }

        let conn_params = match &args[0] {
            Value::Table(t) => t,
            _ => {
                return Err(mlua::Error::external(
                    "First argument must be a table with connection parameters",
                ))
            }
        };

        let query_str = match &args[1] {
            Value::String(s) => s.to_string_lossy(),
            _ => {
                return Err(mlua::Error::external(
                    "Second argument must be a string (SQL query)",
                ))
            }
        };

        // Extract parameters (if any)
        let params: Vec<Value> = args.iter().skip(2).cloned().collect();

        // Use smol to run the async operation synchronously
        smol::block_on(execute_query(lua, conn_params, &query_str, params))
    })?;

    // Function to execute a statement (INSERT, UPDATE, DELETE, etc.)
    let execute_fn = lua.create_function(|_, args: mlua::Variadic<Value>| {
        if args.len() < 2 {
            return Err(mlua::Error::external(
                "sql.execute requires at least 2 arguments: connection_params and query_string",
            ));
        }

        let conn_params = match &args[0] {
            Value::Table(t) => t,
            _ => {
                return Err(mlua::Error::external(
                    "First argument must be a table with connection parameters",
                ))
            }
        };

        let query_str = match &args[1] {
            Value::String(s) => s.to_string_lossy(),
            _ => {
                return Err(mlua::Error::external(
                    "Second argument must be a string (SQL query)",
                ))
            }
        };

        // Extract parameters (if any)
        let params: Vec<Value> = args.iter().skip(2).cloned().collect();

        // Use smol to run the async operation synchronously
        smol::block_on(execute_statement(conn_params, &query_str, params))
    })?;

    // Function to check if we can connect to the database
    let test_connection_fn = lua.create_function(|_, conn_params: Table| {
        // Don't capture conn_params directly in the async block
        // Create a connection string first (doesn't involve async)
        let conn_str = match build_connection_string(&conn_params) {
            Ok(str) => str,
            Err(err) => {
                error!("Database connection string error: {}", err);
                return Ok(false);
            }
        };

        // Create a unique key for the connection
        let conn_key = match create_connection_key(&conn_params) {
            Ok(key) => key,
            Err(err) => {
                error!("Failed to create connection key: {}", err);
                return Ok(false);
            }
        };

        // Check if we already have a pool for this connection
        let existing_pool = {
            let pools = DB_POOLS.lock().unwrap();
            pools.get(&conn_key).cloned()
        };

        smol::block_on(async {
            if let Some(pool) = existing_pool {
                // We already have a pool, test it with a simple query
                match pool.execute("SELECT 1").await {
                    Ok(_) => Ok(true),
                    Err(err) => {
                        error!("Database connection error (existing pool): {}", err);
                        Ok(false)
                    }
                }
            } else {
                // We need to create a new pool
                sqlx::any::install_default_drivers();
                match AnyPoolOptions::new()
                    .max_connections(1)
                    .connect(&conn_str)
                    .await
                {
                    Ok(pool) => {
                        // Store the pool for reuse
                        {
                            let mut pools = DB_POOLS.lock().unwrap();
                            pools.insert(conn_key, pool);
                        }
                        Ok(true)
                    }
                    Err(err) => {
                        error!("Database connection error: {}", err);
                        Ok(false)
                    }
                }
            }
        })
    })?;

    // Function to close a database connection
    let close_connection_fn = lua.create_async_function(|_, conn_params: Table| async move {
        let conn_key = match create_connection_key(&conn_params) {
            Ok(key) => key,
            Err(err) => {
                return Err(mlua::Error::external(format!(
                    "Failed to create connection key: {}",
                    err
                )))
            }
        };

        // Extract the pool and remove it from the map, but drop the mutex guard immediately
        let pool_to_close = {
            let mut pools = DB_POOLS.lock().unwrap();
            pools.remove(&conn_key)
        };

        // Now we can use await without holding the mutex guard
        if let Some(pool) = pool_to_close {
            // Close the pool
            pool.close().await;
            Ok(true)
        } else {
            Ok(false) // No connection to close
        }
    })?;

    // Helper function to create connection parameters
    let create_postgres_connection = lua.create_function(|lua, args: Table| {
        let params = lua.create_table()?;

        params.set("type", "postgres")?;

        if let Ok(host) = args.get::<String>("host") {
            params.set("host", host)?;
        } else {
            params.set("host", "localhost")?;
        }

        if let Ok(port) = args.get::<u16>("port") {
            params.set("port", port)?;
        } else {
            params.set("port", 5432)?;
        }

        if let Ok(username) = args.get::<String>("username") {
            params.set("username", username)?;
        }

        if let Ok(password) = args.get::<String>("password") {
            params.set("password", password)?;
        }

        if let Ok(database) = args.get::<String>("database") {
            params.set("database", database)?;
        }

        Ok(params)
    })?;

    let create_mysql_connection = lua.create_function(|lua, args: Table| {
        let params = lua.create_table()?;

        params.set("type", "mysql")?;

        if let Ok(host) = args.get::<String>("host") {
            params.set("host", host)?;
        } else {
            params.set("host", "localhost")?;
        }

        if let Ok(port) = args.get::<u16>("port") {
            params.set("port", port)?;
        } else {
            params.set("port", 3306)?;
        }

        if let Ok(username) = args.get::<String>("username") {
            params.set("username", username)?;
        }

        if let Ok(password) = args.get::<String>("password") {
            params.set("password", password)?;
        }

        if let Ok(database) = args.get::<String>("database") {
            params.set("database", database)?;
        }

        Ok(params)
    })?;

    let create_sqlite_connection = lua.create_function(|lua, path: String| {
        let params = lua.create_table()?;
        params.set("type", "sqlite")?;
        params.set("path", path)?;

        Ok(params)
    })?;

    sql.set("query", query_fn)?;
    sql.set("execute", execute_fn)?;
    sql.set("testConnection", test_connection_fn)?;
    sql.set("closeConnection", close_connection_fn)?;

    // Connection creation helper functions
    sql.set("postgres", create_postgres_connection)?;
    sql.set("mysql", create_mysql_connection)?;
    sql.set("sqlite", create_sqlite_connection)?;

    Ok(sql)
}
