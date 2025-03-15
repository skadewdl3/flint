use mlua::{Lua, Result as LuaResult, Table, Value, Variadic};

pub fn md_helpers(lua: &Lua) -> LuaResult<Table> {
    let module = lua.create_table()?;

    module.set(
        "link",
        lua.create_function(|_, (name, url): (String, String)| Ok(format!("[{}]({})", name, url)))?,
    )?;

    module.set(
        "text",
        lua.create_function(|_, parts: Variadic<String>| Ok(parts.join(" ")))?,
    )?;

    module.set(
        "bold",
        lua.create_function(|_, text: String| Ok(format!("**{}**", text)))?,
    )?;

    module.set(
        "italic",
        lua.create_function(|_, text: String| Ok(format!("*{}*", text)))?,
    )?;

    module.set(
        "underline",
        lua.create_function(|_, text: String| Ok(format!("<u>{}</u>", text)))?,
    )?;

    for i in 1..=6 {
        let header_function =
            lua.create_function(move |_, text: String| Ok(format!("{} {}", "#".repeat(i), text)))?;
        module.set(format!("h{}", i), header_function)?;
    }

    module.set(
        "ol",
        lua.create_function(|_, items: Vec<String>| {
            Ok(items
                .into_iter()
                .enumerate()
                .map(|(i, item)| format!("{}. {}", i + 1, item))
                .collect::<Vec<_>>()
                .join("\n"))
        })?,
    )?;

    module.set(
        "ul",
        lua.create_function(|_, items: Vec<String>| {
            Ok(items
                .into_iter()
                .map(|item| format!("- {}", item))
                .collect::<Vec<_>>()
                .join("\n"))
        })?,
    )?;

    module.set(
        "code",
        lua.create_function(
            |_, (language, code): (Option<String>, String)| match language {
                Some(lang) => Ok(format!("```{}\n{}\n```", lang, code)),
                None => Ok(format!("```\n{}\n```", code)),
            },
        )?,
    )?;

    module.set(
        "newline",
        lua.create_function(|_, _: ()| Ok("\n".to_string()))?,
    )?;

    module.set(
        "table",
        lua.create_function(|_, table: Table| {
            let mut rows = Vec::new();
            let mut headers = Vec::new();

            for pair in table.pairs::<Value, Table>() {
                let (_, row) = pair?;
                let mut row_values = Vec::new();
                for pair in row.pairs::<String, Value>() {
                    let (key, value) = pair?;
                    if headers.iter().all(|h| h != &key) {
                        headers.push(key.clone());
                    }
                    row_values.push(value.to_string()?);
                }
                rows.push(row_values);
            }

            let header_row = headers.join(" | ");
            let separator = headers
                .iter()
                .map(|_| "---")
                .collect::<Vec<_>>()
                .join(" | ");
            let body = rows
                .into_iter()
                .map(|row| row.join(" | "))
                .collect::<Vec<_>>()
                .join("\n");

            Ok(format!("{}\n{}\n{}", header_row, separator, body))
        })?,
    )?;

    Ok(module)
}
