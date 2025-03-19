use csv::ReaderBuilder;
use flint_utils::Result;
use mlua::{Lua, Table};
use std::fs::File;
use std::io::BufReader;

pub fn csv_helpers(lua: &Lua) -> Result<Table> {
    let csv_module = lua.create_table()?;

    let csv_read = lua.create_function(|lua, file_path: String| {
        let file = File::open(file_path)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to open file: {}", e)))?;
        let mut reader = ReaderBuilder::new().from_reader(BufReader::new(file));

        let result_table = lua.create_table()?;

        // Extract headers
        let header_vec: Vec<_> = {
            reader
                .headers()
                .map_err(|e| mlua::Error::RuntimeError(format!("CSV header error: {}", e)))?
                .iter()
                .map(String::from)
                .collect()
        };

        for (i, record) in reader.records().enumerate() {
            let record =
                record.map_err(|e| mlua::Error::RuntimeError(format!("CSV parse error: {}", e)))?;
            let row_table = lua.create_table()?;

            for (j, field) in record.iter().enumerate() {
                row_table.raw_set(header_vec.get(j).unwrap().to_string(), field)?;
            }

            result_table.raw_set(i + 1, row_table)?;
        }

        Ok(result_table)
    })?;

    csv_module.set("read", csv_read)?;

    Ok(csv_module)
}
