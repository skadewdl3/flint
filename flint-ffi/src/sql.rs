use sea_orm::ConnectionTrait;
use serde_json::Map;
use std::sync::Arc;

use mlua::{
    Lua, LuaSerdeExt, Result as LuaResult, Table, UserData, UserDataMethods, Value as LuaValue,
    Variadic,
};
use sea_orm::{Database, DatabaseConnection, DbErr, Statement, Value as SeaValue};

pub struct DbConnection {
    conn: Arc<DatabaseConnection>,
}

impl DbConnection {
    async fn new(url: &str) -> Result<Self, DbErr> {
        let conn = Database::connect(url).await?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }
}

impl UserData for DbConnection {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method(
            "execute",
            |lua, this, args: Variadic<mlua::Value>| async move {
                let query: String = lua.from_value(args[0].clone())?;
                let params = args[1..].iter();
                let mut new_params: Vec<SeaValue> = Vec::new();
                for param in params {
                    let val = SeaSqlValue::from_lua(&param, &lua)?;
                    new_params.push(val);
                }

                match this
                    .conn
                    .execute(Statement::from_sql_and_values(
                        this.conn.get_database_backend(),
                        query,
                        new_params,
                    ))
                    .await
                {
                    Ok(result) => {
                        let lua_result = lua.create_table()?;
                        lua_result.set("rows_affected", result.rows_affected())?;
                        lua_result.set("last_insert_id", result.last_insert_id().to_string())?;
                        Ok(lua_result)
                    }
                    Err(e) => Err(mlua::Error::RuntimeError(format!("Database error: {}", e))),
                }
            },
        );

        methods.add_async_method(
            "query",
            |lua, this, args: Variadic<mlua::Value>| async move {
                let query: String = lua.from_value(args[0].clone())?;
                let params = args[1..].iter();
                let params = params.map(|v| SeaSqlValue::from_lua(&v, &lua).unwrap());

                match this
                    .conn
                    .query_all(Statement::from_sql_and_values(
                        this.conn.get_database_backend(),
                        query,
                        params,
                    ))
                    .await
                {
                    Ok(result) => {
                        use serde_json::Value as JsonValue;
                        let mut json_results: Vec<JsonValue> = Vec::new();

                        for row in result.iter() {
                            let mut json_row = Map::new();

                            // Iterate over each column by index
                            for (idx, col_name) in row.column_names().iter().enumerate() {
                                // Attempt to retrieve the value at the current index
                                if let Ok(val) = row.try_get_by_index::<i64>(idx) {
                                    json_row.insert(
                                        col_name.to_string(),
                                        JsonValue::Number(
                                            serde_json::Number::from_i128(val as i128).unwrap(),
                                        ),
                                    );
                                } else if let Ok(val) = row.try_get_by_index::<f64>(idx) {
                                    json_row.insert(
                                        col_name.to_string(),
                                        JsonValue::Number(
                                            serde_json::Number::from_f64(val).unwrap(),
                                        ),
                                    );
                                } else if let Ok(val) = row.try_get_by_index::<String>(idx) {
                                    json_row.insert(col_name.to_string(), JsonValue::String(val));
                                } else if let Ok(val) = row.try_get_by_index::<bool>(idx) {
                                    json_row.insert(col_name.to_string(), JsonValue::Bool(val));
                                } else {
                                    json_row.insert(col_name.to_string(), JsonValue::Null);
                                }
                            }

                            // Convert the map into a JsonValue and add it to the results vector
                            json_results.push(JsonValue::Object(json_row));
                        }

                        let lua_result: mlua::Value = lua.to_value(&json_results)?;

                        Ok(lua_result)
                    }
                    Err(e) => Err(mlua::Error::RuntimeError(format!("Database error: {}", e))),
                }
            },
        );

        methods.add_method("execute_sync", |lua, this, args: Variadic<mlua::Value>| {
            let query: String = lua.from_value(args[0].clone())?;
            let params = args[1..].iter();
            let params = params.map(|v| SeaSqlValue::from_lua(&v, &lua).unwrap());

            let result = smol::block_on(this.conn.execute(Statement::from_sql_and_values(
                this.conn.get_database_backend(),
                query,
                params,
            )));

            match result {
                Ok(result) => {
                    let lua_result = lua.create_table()?;
                    lua_result.set("rows_affected", result.rows_affected())?;
                    lua_result.set("last_insert_id", result.last_insert_id().to_string())?;
                    Ok(lua_result)
                }
                Err(e) => Err(mlua::Error::RuntimeError(format!("Database error: {}", e))),
            }
        });

        methods.add_method("query_sync", |lua, this, args: Variadic<mlua::Value>| {
            let query: String = lua.from_value(args[0].clone())?;
            let params = args[1..].iter();
            let params = params.map(|v| SeaSqlValue::from_lua(&v, &lua).unwrap());

            let result = smol::block_on(this.conn.query_all(Statement::from_sql_and_values(
                this.conn.get_database_backend(),
                query,
                params,
            )));

            match result {
                Ok(result) => {
                    use serde_json::Value as JsonValue;
                    let mut json_results: Vec<JsonValue> = Vec::new();

                    for row in result.iter() {
                        let mut json_row = Map::new();

                        for (idx, col_name) in row.column_names().iter().enumerate() {
                            if let Ok(val) = row.try_get_by_index::<i64>(idx) {
                                json_row.insert(
                                    col_name.to_string(),
                                    JsonValue::Number(
                                        serde_json::Number::from_i128(val as i128).unwrap(),
                                    ),
                                );
                            } else if let Ok(val) = row.try_get_by_index::<f64>(idx) {
                                json_row.insert(
                                    col_name.to_string(),
                                    JsonValue::Number(serde_json::Number::from_f64(val).unwrap()),
                                );
                            } else if let Ok(val) = row.try_get_by_index::<String>(idx) {
                                json_row.insert(col_name.to_string(), JsonValue::String(val));
                            } else if let Ok(val) = row.try_get_by_index::<bool>(idx) {
                                json_row.insert(col_name.to_string(), JsonValue::Bool(val));
                            } else {
                                json_row.insert(col_name.to_string(), JsonValue::Null);
                            }
                        }

                        json_results.push(JsonValue::Object(json_row));
                    }

                    let lua_result: mlua::Value = lua.to_value(&json_results)?;
                    Ok(lua_result)
                }
                Err(e) => Err(mlua::Error::RuntimeError(format!("Database error: {}", e))),
            }
        })
    }
}

struct SeaSqlValue;

pub fn lua_value_to_json_value(value: mlua::Value) -> mlua::Result<sea_orm::query::JsonValue> {
    use sea_orm::query::JsonValue;
    use serde_json::Number;
    Ok(match value {
        LuaValue::Nil => JsonValue::Null,
        LuaValue::Boolean(b) => JsonValue::Bool(b),
        LuaValue::Integer(i) => JsonValue::Number(Number::from(i)),
        LuaValue::Number(n) => JsonValue::Number(Number::from_f64(n).ok_or_else(|| {
            mlua::Error::FromLuaConversionError {
                from: "Number",
                to: "JsonValue".to_string(),
                message: Some("Invalid f64 value".to_string()),
            }
        })?),
        LuaValue::String(s) => JsonValue::String(s.to_str()?.to_string()),
        _ => {
            return Err(mlua::Error::FromLuaConversionError {
                from: "LuaValue",
                to: "JsonValue".to_string(),
                message: Some("Unsupported Lua type".to_string()),
            });
        }
    })
}

impl SeaSqlValue {
    pub fn from_lua(value: &mlua::Value, lua: &Lua) -> LuaResult<SeaValue> {
        if let mlua::Value::Table(tbl) = value {
            let kind: String = tbl.get("kind")?;
            let value: mlua::Value = tbl.get("value")?;

            match kind.as_str() {
                "bool" => Ok(match lua.from_value::<bool>(value) {
                    Ok(v) => SeaValue::Bool(Some(v)),
                    Err(_) => SeaValue::Bool(None),
                }),
                "tiny_int" => Ok(match lua.from_value::<i8>(value) {
                    Ok(v) => SeaValue::TinyInt(Some(v)),
                    Err(_) => SeaValue::TinyInt(None),
                }),
                "small_int" => Ok(match lua.from_value::<i16>(value) {
                    Ok(v) => SeaValue::SmallInt(Some(v)),
                    Err(_) => SeaValue::SmallInt(None),
                }),
                "int" => Ok(match lua.from_value::<i32>(value) {
                    Ok(v) => SeaValue::Int(Some(v)),
                    Err(_) => SeaValue::Int(None),
                }),
                "big_int" => Ok(match lua.from_value::<i64>(value) {
                    Ok(v) => SeaValue::BigInt(Some(v)),
                    Err(_) => SeaValue::BigInt(None),
                }),
                "tiny_unsigned" => Ok(match lua.from_value::<u8>(value) {
                    Ok(v) => SeaValue::TinyUnsigned(Some(v)),
                    Err(_) => SeaValue::TinyUnsigned(None),
                }),
                "small_unsigned" => Ok(match lua.from_value::<u16>(value) {
                    Ok(v) => SeaValue::SmallUnsigned(Some(v)),
                    Err(_) => SeaValue::SmallUnsigned(None),
                }),
                "unsigned" => Ok(match lua.from_value::<u32>(value) {
                    Ok(v) => SeaValue::Unsigned(Some(v)),
                    Err(_) => SeaValue::Unsigned(None),
                }),
                "big_unsigned" => Ok(match lua.from_value::<u64>(value) {
                    Ok(v) => SeaValue::BigUnsigned(Some(v)),
                    Err(_) => SeaValue::BigUnsigned(None),
                }),
                "float" => Ok(match lua.from_value::<f32>(value) {
                    Ok(v) => SeaValue::Float(Some(v)),
                    Err(_) => SeaValue::Float(None),
                }),
                "bouble" => Ok(match lua.from_value::<f64>(value) {
                    Ok(v) => SeaValue::Double(Some(v)),
                    Err(_) => SeaValue::Double(None),
                }),
                "string" => Ok(match lua.from_value::<String>(value) {
                    Ok(s) => SeaValue::String(Some(Box::new(s))),
                    Err(_) => SeaValue::String(None),
                }),
                "char" => Ok(match lua.from_value::<char>(value) {
                    Ok(v) => SeaValue::Char(Some(v)),
                    Err(_) => SeaValue::Char(None),
                }),
                "bytes" => Ok(match lua.from_value::<Box<Vec<u8>>>(value) {
                    Ok(v) => SeaValue::Bytes(Some(v)),
                    Err(_) => SeaValue::Bytes(None),
                }),
                "json" => Ok(match lua_value_to_json_value(value) {
                    Ok(v) => SeaValue::Json(Some(Box::new(v))),
                    Err(_) => SeaValue::Json(None),
                }),
                _ => Err(mlua::Error::FromLuaConversionError {
                    from: "LuaSqlValue",
                    to: "SeaValue".to_string(),
                    message: Some("Unsupported value kind".to_string()),
                }),
            }
        } else {
            return Err(mlua::Error::FromLuaConversionError {
                from: "LuaSqlValue",
                to: "SeaValue".to_string(),
                message: Some("Unsupported value kind".to_string()),
            });
        }
    }
}

pub fn sql_helpers(lua: &Lua) -> LuaResult<Table> {
    let sql = lua.create_table()?;

    sql.set(
        "connect",
        lua.create_async_function(|_, url: String| async move {
            let x = DbConnection::new(&url).await;
            match x {
                Ok(conn) => Ok(conn),
                Err(e) => Err(mlua::Error::RuntimeError(format!(
                    "Connection error: {}",
                    e
                ))),
            }
        })?,
    )?;

    let postgres_conn = lua.create_function(|_, params: Table| {
        let username: String = params.get("username")?;
        let password: String = params.get("password")?;
        let host: String = params.get("host")?;
        let port: String = params.get("port")?;
        let database: String = params.get("database")?;
        let conn_string = format!(
            "postgres://{}:{}@{}:{}/{}",
            username, password, host, port, database
        );
        Ok(conn_string)
    })?;

    let mysql_conn = lua.create_function(|_, params: Table| {
        let username: String = params.get("username")?;
        let password: String = params.get("password")?;
        let host: String = params.get("host")?;
        let database: String = params.get("database")?;
        let port: String = params.get("port")?;
        let conn_string = format!(
            "mysql://{}:{}@{}:{}/{}",
            username, password, host, port, database
        );
        Ok(conn_string)
    })?;

    // Function to generate SQLite connection string
    let sqlite_conn = lua.create_function(|_, params: Table| {
        let path: String = params.get("path")?;
        let conn_string = format!("sqlite://{}", path);
        Ok(conn_string)
    })?;

    sql.set("postgres", postgres_conn)?;
    sql.set("mysql", mysql_conn)?;
    sql.set("sqlite", sqlite_conn)?;

    let cases: Vec<String> = vec![
        "bool",
        "tiny_int",
        "small_int",
        "int",
        "big_int",
        "tiny_unsigned",
        "small_unsigned",
        "unsigned",
        "big_unsigned",
        "float",
        "double",
        "string",
        "char",
        "bytes",
        "json",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    for kind in cases {
        sql.set(
            kind.clone(),
            lua.create_function(move |lua, value: mlua::Value| {
                let tbl = lua.create_table()?;
                tbl.set("kind", kind.clone())?;
                tbl.set("value", value)?;
                Ok(tbl)
            })?,
        )?;
    }

    Ok(sql)
}
