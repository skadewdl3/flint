use mlua::{Function, Lua, Result as LuaResult, Table, Value, Variadic};

pub fn indent_js_object(input: &str) -> String {
    let mut result = String::new();
    let mut indent_level = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = input.chars().peekable();

    // Helper function to add proper indentation
    let add_indent = |level: usize| -> String { "    ".repeat(level) };

    while let Some(c) = chars.next() {
        match c {
            '"' | '\'' => {
                if !escaped {
                    in_string = !in_string;
                }
                result.push(c);
            }
            '\\' => {
                result.push(c);
                escaped = !escaped;
                continue;
            }
            '{' | '[' => {
                result.push(c);
                if !in_string {
                    indent_level += 1;

                    // Check if the next non-whitespace char is closing bracket
                    let mut temp_chars = chars.clone();
                    let mut next_meaningful = None;
                    while let Some(next_c) = temp_chars.next() {
                        if !next_c.is_whitespace() {
                            next_meaningful = Some(next_c);
                            break;
                        }
                    }

                    if next_meaningful != Some('}') && next_meaningful != Some(']') {
                        result.push('\n');
                        result.push_str(&add_indent(indent_level));
                    }
                }
            }
            '}' | ']' => {
                if !in_string {
                    indent_level = indent_level.saturating_sub(1);

                    // Add newline before closing brace unless object is empty
                    let last_non_whitespace = result.trim_end().chars().last();
                    if last_non_whitespace != Some('{') && last_non_whitespace != Some('[') {
                        result.push('\n');
                        result.push_str(&add_indent(indent_level));
                    }
                }
                result.push(c);
            }
            ',' => {
                result.push(c);
                if !in_string {
                    result.push('\n');
                    result.push_str(&add_indent(indent_level));
                }
            }
            ':' => {
                result.push(c);
                if !in_string {
                    result.push(' ');
                }
            }
            _ => {
                if c.is_whitespace() && !in_string {
                    // Skip whitespace except in strings
                    // but preserve space after commas which we handle separately
                    let last_char = result.chars().last();
                    if !(last_char == Some('\n') || last_char == Some(' ')) {
                        result.push(' ');
                    }
                } else {
                    result.push(c);
                }
                escaped = false;
            }
        }
    }

    result
}

// Helper trait to add padding method
trait PadToWidth {
    fn pad_to_width(&self, width: usize) -> String;
}

impl PadToWidth for String {
    fn pad_to_width(&self, width: usize) -> String {
        if self.len() >= width {
            self.clone()
        } else {
            format!("{}{}", self, " ".repeat(width - self.len()))
        }
    }
}

pub fn import_helpers(lua: &Lua) -> LuaResult<Table> {
    // Create imports subtable
    let imports_table = lua.create_table()?;

    // Implement named imports function with new syntax
    let named_fn = lua.create_function(|ctx, (name, from): (String, String)| {
        let obj = ctx.create_table()?;
        obj.set("type", "named")?;
        obj.set("from", from)?;

        // Create items array with just the single item
        let items = ctx.create_table()?;
        items.set(1, name.clone())?;
        obj.set("items", items)?;

        // Create imports array
        let imports = ctx.create_table()?;
        let import_entry = ctx.create_table()?;
        import_entry.set("name", name.clone())?;
        import_entry.set("alias", Value::Nil)?;
        imports.set(1, import_entry)?;
        obj.set("imports", imports)?;

        // Set metatable with __tostring and __js_import marker
        let mt = ctx.create_table()?;
        mt.set(
            "__tostring",
            ctx.create_function(move |_, _: ()| Ok(name.clone()))?,
        )?;
        mt.set("__js_import", true)?;

        obj.set_metatable(Some(mt));
        Ok(obj)
    })?;
    imports_table.set("named", named_fn)?;

    // Implement alias imports function with new syntax
    let alias_fn = lua.create_function(|ctx, (name, alias, from): (String, String, String)| {
        let obj = ctx.create_table()?;
        obj.set("type", "alias")?;
        obj.set("from", from)?;

        // Create items array with name and alias
        let items = ctx.create_table()?;
        items.set(1, name.clone())?;
        items.set(2, alias.clone())?;
        obj.set("items", items)?;

        // Create imports array
        let imports = ctx.create_table()?;
        let import_entry = ctx.create_table()?;
        import_entry.set("name", name)?;
        import_entry.set("alias", alias.clone())?;
        imports.set(1, import_entry)?;
        obj.set("imports", imports)?;

        // Set metatable with __tostring and __js_import marker
        let mt = ctx.create_table()?;
        mt.set(
            "__tostring",
            ctx.create_function(move |_, _: ()| Ok(alias.clone()))?,
        )?;
        mt.set("__js_import", true)?;

        obj.set_metatable(Some(mt));
        Ok(obj)
    })?;
    imports_table.set("alias", alias_fn)?;

    // Implement default imports function (unchanged)
    let default_fn = lua.create_function(|ctx, (name, from): (String, String)| {
        let obj = ctx.create_table()?;
        obj.set("type", "default")?;
        obj.set("name", name.clone())?;
        obj.set("from", from)?;

        // Set metatable with __tostring and __js_import marker
        let mt = ctx.create_table()?;
        let name_clone = name.clone();
        mt.set(
            "__tostring",
            ctx.create_function(move |_, _: ()| Ok(name_clone.clone()))?,
        )?;
        mt.set("__js_import", true)?;

        obj.set_metatable(Some(mt));
        Ok(obj)
    })?;
    imports_table.set("default", default_fn)?;

    // Rest of the function remains unchanged...
    // Implement merge imports function
    let merge_fn = lua.create_function(|ctx, args: Variadic<Table>| {
        let args_len = args.len();

        let merged = ctx.create_table()?;
        let by_module = ctx.create_table()?;
        let defaults = ctx.create_table()?;

        merged.set("byModule", by_module.clone())?;
        merged.set("defaults", defaults.clone())?;

        for i in 0..args_len {
            let import: &Table = args.get(i).unwrap();
            let import_type: String = import.get("type")?;

            if import_type == "default" {
                let from: String = import.get("from")?;
                let name: String = import.get("name")?;
                defaults.set(from, name)?;
            } else {
                let from: String = import.get("from")?;
                let imports: Table = import.get("imports")?;

                let module_imports: Table = match by_module.get(from.clone())? {
                    Value::Table(t) => t,
                    _ => {
                        let new_table = ctx.create_table()?;
                        by_module.set(from.clone(), new_table.clone())?;
                        new_table
                    }
                };

                let imports_len: i32 = imports.len()?;
                let current_len: i32 = module_imports.len()?;

                for j in 1..=imports_len {
                    let import_item: Table = imports.get(j)?;
                    module_imports.set(current_len + j, import_item)?;
                }
            }
        }

        // Create metatable with __tostring method for merged imports
        let mt = ctx.create_table()?;
        mt.set(
            "__tostring",
            ctx.create_function(move |_, merged: Table| {
                let mut result = Vec::new();
                let by_module: Table = merged.get("byModule")?;
                let defaults: Table = merged.get("defaults")?;

                // Process named and aliased imports
                for pair in by_module.pairs::<String, Table>() {
                    let (module_name, imports) = pair?;
                    let mut import_parts = Vec::new();

                    for i in 1..=imports.len()? {
                        let import: Table = imports.get(i)?;
                        let name: String = import.get("name")?;
                        let alias: Value = import.get("alias")?;

                        match alias {
                            Value::String(alias_str) => {
                                // If we have an alias, use "name as alias" format
                                let temp = alias_str.to_string_lossy();
                                import_parts.push(format!("{} as {}", name, temp));
                            }
                            Value::Nil => {
                                // If no alias, just use the name
                                import_parts.push(name);
                            }
                            _ => continue,
                        }
                    }

                    if !import_parts.is_empty() {
                        result.push(format!(
                            "import {{ {} }} from \"{}\"",
                            import_parts.join(", "),
                            module_name
                        ));
                    }
                }

                // Process default imports
                for pair in defaults.pairs::<String, String>() {
                    let (module_name, name) = pair?;
                    result.push(format!("import {} from \"{}\"", name, module_name));
                }

                // Join all imports with newlines
                Ok(result.join("\n"))
            })?,
        )?;

        merged.set_metatable(Some(mt));
        Ok(merged)
    })?;
    imports_table.set("merge", merge_fn)?;

    Ok(imports_table)
}

pub fn array_helpers(lua: &Lua) -> LuaResult<Function> {
    lua.create_function(|ctx, args: Variadic<Value>| {
        let arr = ctx.create_table()?;
        let args_len = args.len();

        for i in 0..args_len {
            let item = args.get(i).unwrap();
            arr.set(i + 1, item)?;
        }

        // Create metatable with __tostring method
        let mt = ctx.create_table()?;
        mt.set(
            "__tostring",
            ctx.create_function(|ctx, arr: Table| {
                let len = arr.len()?;
                let mut parts = Vec::new();

                for i in 1..=len {
                    let item: Value = arr.get(i)?;
                    let serialize_fn: mlua::Function = ctx.globals().get("__js_serialize_value")?;
                    parts.push(serialize_fn.call::<String>(item)?);
                }

                // Format array with proper spacing for better readability
                if parts.is_empty() {
                    Ok("[]".to_string())
                } else if parts.len() <= 3 && parts.iter().all(|p| p.len() < 30) {
                    // For small arrays, keep them on one line
                    Ok(format!("[ {} ]", parts.join(", ")))
                } else {
                    // For larger arrays, format with newlines and indentation
                    Ok(format!("[\n  {}\n]", parts.join(",\n  ")))
                }
            })?,
        )?;

        arr.set_metatable(Some(mt));
        Ok(arr)
    })
}

pub fn object_helpers(lua: &Lua) -> LuaResult<Function> {
    lua.create_function(|ctx, args: Table| {
        let obj = ctx.create_table()?;

        // Copy key-value pairs from args to obj
        for pair in args.pairs::<Value, Value>() {
            let (key, value) = pair?;
            obj.set(key, value)?;
        }

        // Create metatable with __tostring method
        let mt = ctx.create_table()?;
        mt.set(
            "__tostring",
            ctx.create_function(|ctx, obj: Table| {
                let mut parts = Vec::new();
                let mut longest_key = 0;

                // First pass to determine the longest key for alignment
                for pair in obj.pairs::<Value, Value>() {
                    if let (Value::String(s), _) = pair? {
                        let key_len = s.to_string_lossy().len();
                        if key_len > longest_key {
                            longest_key = key_len;
                        }
                    }
                }

                // Second pass to format the entries
                for pair in obj.pairs::<Value, Value>() {
                    let (key, value) = pair?;

                    // Format the key
                    let key_str = match key {
                        Value::String(s) => {
                            let key_str = s.to_string_lossy();
                            // Check if key is a valid JS identifier
                            let is_valid_identifier = !key_str.is_empty()
                                && key_str.chars().next().unwrap().is_alphabetic()
                                && key_str.chars().all(|c| c.is_alphanumeric() || c == '_');

                            if is_valid_identifier {
                                key_str.to_string()
                            } else {
                                // Quote keys that aren't valid identifiers
                                format!("\"{}\"", key_str.replace("\"", "\\\""))
                            }
                        }
                        _ => continue, // Skip non-string keys
                    };

                    // Format the value using the serializer function
                    let serialize_fn: mlua::Function = ctx.globals().get("__js_serialize_value")?;
                    let value_str = serialize_fn.call::<String>(value)?;

                    parts.push(format!(
                        "  {}: {}",
                        key_str.pad_to_width(longest_key),
                        value_str
                    ));
                }

                if parts.is_empty() {
                    Ok("{}".to_string())
                } else {
                    Ok(format!("{{\n{}\n}}", parts.join(",\n")))
                }
            })?,
        )?;

        obj.set_metatable(Some(mt));
        Ok(obj)
    })
}

pub fn serialize_helpers(lua: &Lua) -> LuaResult<Function> {
    lua.create_function(|ctx, value: Value| {
        match value {
            Value::String(s) => {
                let escaped = s
                    .to_string_lossy()
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\"")
                    .replace("\n", "\\n")
                    .replace("\r", "\\r")
                    .replace("\t", "\\t");
                Ok(format!("\"{}\"", escaped))
            }
            Value::Integer(n) => Ok(n.to_string()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            Value::Table(t) => {
                // Check if it's an import object (has __js_import metatable)
                if let Some(mt) = t.metatable() {
                    if mt.get::<bool>("__js_import")? {
                        // Get the import identifier using __tostring
                        let to_string: mlua::Function = mt.get("__tostring")?;
                        Ok(to_string.call(t)?)
                    } else if mt.get::<bool>("__js_null")? {
                        Ok("null".to_string())
                    } else if mt.get::<bool>("__js_undefined")? {
                        Ok("undefined".to_string())
                    } else if let Ok(to_string) = mt.get::<mlua::Function>("__tostring") {
                        // For other tables with __tostring, use that
                        Ok(to_string.call(t)?)
                    } else {
                        // Directly check if it's an array-like table
                        let len: i32 = t.len()?;
                        if len > 0 {
                            // Serialize as array
                            let mut parts = Vec::new();
                            for i in 1..=len {
                                let item: Value = t.get(i)?;
                                let serialize_fn: mlua::Function =
                                    ctx.globals().get("__js_serialize_value")?;
                                parts.push(serialize_fn.call::<String>(item)?);
                            }
                            Ok(format!("[{}]", parts.join(", ")))
                        } else {
                            // Serialize as object
                            let mut parts = Vec::new();
                            for pair in t.pairs::<Value, Value>() {
                                let (key, value) = pair?;

                                // Format the key
                                let key_str = match key {
                                    Value::String(s) => {
                                        let key_str = s.to_string_lossy();
                                        // Check if key is a valid JS identifier
                                        let is_valid_identifier = !key_str.is_empty()
                                            && key_str.chars().next().unwrap().is_alphabetic()
                                            && key_str
                                                .chars()
                                                .all(|c| c.is_alphanumeric() || c == '_');

                                        if is_valid_identifier {
                                            key_str.to_string()
                                        } else {
                                            // Quote keys that aren't valid identifiers
                                            format!("\"{}\"", key_str.replace("\"", "\\\""))
                                        }
                                    }
                                    _ => continue, // Skip non-string keys
                                };

                                // Format the value using the serializer function
                                let serialize_fn: mlua::Function =
                                    ctx.globals().get("__js_serialize_value")?;
                                let value_str = serialize_fn.call::<String>(value)?;

                                parts.push(format!("{}: {}", key_str, value_str));
                            }
                            Ok(format!("{{ {} }}", parts.join(", ")))
                        }
                    }
                } else {
                    // Same as above, but for tables without metatable
                    let len: i32 = t.len()?;
                    if len > 0 {
                        // Serialize as array
                        let mut parts = Vec::new();
                        for i in 1..=len {
                            let item: Value = t.get(i)?;
                            let serialize_fn: mlua::Function =
                                ctx.globals().get("__js_serialize_value")?;
                            parts.push(serialize_fn.call::<String>(item)?);
                        }
                        Ok(format!("[{}]", parts.join(", ")))
                    } else {
                        // Serialize as object
                        let mut parts = Vec::new();
                        for pair in t.pairs::<Value, Value>() {
                            let (key, value) = pair?;

                            // Format the key
                            let key_str = match key {
                                Value::String(s) => {
                                    let key_str = s.to_string_lossy();
                                    // Check if key is a valid JS identifier
                                    let is_valid_identifier = !key_str.is_empty()
                                        && key_str.chars().next().unwrap().is_alphabetic()
                                        && key_str.chars().all(|c| c.is_alphanumeric() || c == '_');

                                    if is_valid_identifier {
                                        key_str.to_string()
                                    } else {
                                        // Quote keys that aren't valid identifiers
                                        format!("\"{}\"", key_str.replace("\"", "\\\""))
                                    }
                                }
                                _ => continue, // Skip non-string keys
                            };

                            // Format the value using the serializer function
                            let serialize_fn: mlua::Function =
                                ctx.globals().get("__js_serialize_value")?;
                            let value_str = serialize_fn.call::<String>(value)?;

                            parts.push(format!("{}: {}", key_str, value_str));
                        }
                        Ok(format!("{{ {} }}", parts.join(", ")))
                    }
                }
            }
            Value::Nil => Ok("null".to_string()),
            _ => Ok("null".to_string()), // Handle other value types
        }
    })
}

pub fn export_helpers(lua: &Lua) -> LuaResult<Table> {
    // Create exports subtable
    let exports_table = lua.create_table()?;

    // Implement default export function
    let default_fn = lua.create_function(|ctx, value: Value| {
        let serialize_fn: mlua::Function = ctx.globals().get("__js_serialize_value")?;
        let serialized = serialize_fn.call::<String>(value)?;

        // Format with proper spacing and newline for readability
        Ok(format!("export default {}", serialized))
    })?;
    exports_table.set("default", default_fn)?;

    // Implement named export function for multiple exports
    let named_fn = lua.create_function(|ctx, exports: Table| {
        let mut parts = Vec::new();
        let mut longest_name = 0;

        // First pass to find the longest export name for alignment
        for pair in exports.pairs::<String, Value>() {
            let (name, _) = pair?;
            if name.len() > longest_name {
                longest_name = name.len();
            }
        }

        // Second pass to format each export with proper alignment
        for pair in exports.pairs::<String, Value>() {
            let (name, value) = pair?;
            let serialize_fn: mlua::Function = ctx.globals().get("__js_serialize_value")?;
            let serialized = serialize_fn.call::<String>(value)?;

            // Pad the name for aligned equals signs
            let padded_name = format!("{}{}", name, " ".repeat(longest_name - name.len()));
            parts.push(format!("export const {} = {}", padded_name, serialized));
        }

        Ok(parts.join("\n"))
    })?;
    exports_table.set("named", named_fn)?;

    Ok(exports_table)
}

pub fn function_helpers(lua: &Lua) -> LuaResult<Table> {
    let fn_table = lua.create_table()?;

    let call_fn = lua.create_function(|ctx, args: Variadic<Value>| {
        // We need at least one argument (the function to call)
        if args.len() == 0 {
            return Err(mlua::Error::RuntimeError(
                "js.fn.call requires at least a function argument".to_string(),
            ));
        }

        let function = args.get(0).unwrap().clone();
        let obj = ctx.create_table()?;

        // Store the function and arguments for later use
        obj.set("function", function.clone())?;

        let call_args = ctx.create_table()?;
        for i in 1..args.len() {
            call_args.set(i, args.get(i).unwrap())?;
        }
        obj.set("args", call_args)?;

        // Set metatable with __tostring for function call serialization
        let mt = ctx.create_table()?;
        mt.set(
            "__tostring",
            ctx.create_function(|ctx, obj: Table| {
                let func: Value = obj.get("function")?;
                let args: Table = obj.get("args")?;

                // Get function name via serialization or __tostring
                let func_str = match func {
                    Value::Table(t) => {
                        if let Some(mt) = t.metatable() {
                            if let Ok(to_string) = mt.get::<mlua::Function>("__tostring") {
                                to_string.call::<String>(t)?
                            } else {
                                // Fallback if no __tostring
                                "function".to_string()
                            }
                        } else {
                            // Fallback if no metatable
                            "function".to_string()
                        }
                    }
                    Value::String(s) => s.to_string_lossy().to_string(),
                    _ => "function".to_string(),
                };

                // Format arguments
                let mut arg_parts = Vec::new();
                let args_len: i32 = args.len()?;

                for i in 1..=args_len {
                    let arg: Value = args.get(i)?;
                    let serialize_fn: mlua::Function = ctx.globals().get("__js_serialize_value")?;
                    arg_parts.push(serialize_fn.call::<String>(arg)?);
                }

                // Format as function call
                if arg_parts.is_empty() {
                    Ok(format!("{}()", func_str))
                } else if arg_parts.len() == 1 && arg_parts[0].len() < 60 {
                    // Simple one-argument call
                    Ok(format!("{}({})", func_str, arg_parts[0]))
                } else {
                    // Format multi-line for better readability
                    Ok(format!("{}(\n  {}\n)", func_str, arg_parts.join(",\n  ")))
                }
            })?,
        )?;

        obj.set_metatable(Some(mt));
        Ok(obj)
    })?;

    fn_table.set("call", call_fn)?;
    Ok(fn_table)
}

pub fn js_helpers(lua: &Lua) -> LuaResult<Table> {
    // Create the main js table
    let js_table = lua.create_table()?;

    let imports_table = import_helpers(lua)?;
    js_table.set("imports", &imports_table)?;

    let exports_table = export_helpers(lua)?;
    js_table.set("exports", &exports_table)?;

    let null_value = lua.create_table()?;
    let null_mt = lua.create_table()?;
    null_mt.set("__tostring", lua.create_function(|_, _: ()| Ok("null"))?)?;
    null_value.set_metatable(Some(null_mt));
    js_table.set("null", null_value)?;

    let undefined_value = lua.create_table()?;
    let undefined_mt = lua.create_table()?;
    undefined_mt.set(
        "__tostring",
        lua.create_function(|_, _: ()| Ok("undefined"))?,
    )?;
    undefined_value.set_metatable(Some(undefined_mt));
    js_table.set("undefined", undefined_value)?;

    // Register the serializer as a global function (will be removed later)
    let serialize_value_fn = serialize_helpers(lua)?;
    lua.globals()
        .set("__js_serialize_value", serialize_value_fn)?;

    let array_fn = array_helpers(lua)?;
    js_table.set("array", array_fn)?;

    let object_fn = object_helpers(lua)?;
    js_table.set("object", object_fn)?;

    let fn_table = function_helpers(lua)?;
    js_table.set("fn", fn_table)?;
    // Add pretty-printing function
    let indent_fn = lua.create_function(|_, js_str: String| Ok(indent_js_object(&js_str)))?;
    js_table.set("indent", indent_fn)?;

    Ok(js_table)
}
