use mlua::{Lua, Result as LuaResult, Table, Variadic};
use std::process::{Child, Command as StdCommand, Stdio};
use std::sync::{Arc, Mutex};

pub fn command_helpers(lua: &Lua) -> LuaResult<Table> {
    let cmd_module = lua.create_table()?;
    let processes: Arc<Mutex<Vec<Child>>> = Arc::new(Mutex::new(vec![]));

    let cmd_run = lua.create_function({
        let processes = processes.clone();
        move |lua, args: Variadic<String>| {
            if args.is_empty() {
                return Err(mlua::Error::RuntimeError(
                    "Expected a command to run".into(),
                ));
            }

            let cmd = args.join(" ");

            let command = StdCommand::new(&args[0]).args(&args[1..]).spawn().unwrap();
            let pid = command.id();
            let output = command.wait_with_output().unwrap();

            let result = lua.create_table()?;
            result.set("command", cmd)?;
            result.set("pid", pid)?;
            result.set(
                "output",
                lua.create_function(move |lua, _: ()| {
                    let tbl = lua.create_table()?;
                    tbl.set("success", output.status.success())?;
                    tbl.set("exit_code", output.status.code())?;
                    tbl.set("stdout", String::from_utf8_lossy(&output.stdout))?;
                    tbl.set("stderr", String::from_utf8_lossy(&output.stderr))?;
                    Ok(tbl)
                })?,
            )?;

            Ok(result)
        }
    })?;

    let cmd_run_async = lua.create_function({
        let processes = processes.clone();
        move |lua, args: Variadic<String>| {
            if args.is_empty() {
                return Err(mlua::Error::RuntimeError(
                    "Expected a command to run".into(),
                ));
            }

            let cmd = args.join(" ");
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn({
                let processes = processes.clone();
                move || {
                    let command = StdCommand::new(&args[0])
                        .args(&args[1..])
                        .stdout(Stdio::null()) // Keep it running without blocking
                        .stderr(Stdio::null())
                        .spawn()
                        .expect("Failed to start process");

                    let cmd_id = command.id();
                    tx.send(cmd_id).unwrap(); // Send PID back

                    processes.lock().unwrap().push(command); // Keep process alive
                }
            });

            let pid = rx.recv().unwrap_or(0);

            let result = lua.create_table()?;
            result.set("command", cmd)?;
            result.set("pid", pid)?;
            result.set(
                "output",
                lua.create_async_function(async move |_, ()| -> LuaResult<String> {
                    Ok("You cannot call .output() on an async command".into())
                })?,
            )?;

            Ok(result)
        }
    })?;

    let kill = lua.create_function({
        let processes = processes.clone();
        move |_, pid: u32| {
            let mut res = processes.lock().unwrap();
            if let Some(index) = res.iter().position(|c| c.id() == pid) {
                let mut process = res.remove(index);
                process.kill().ok();
            }
            Ok(())
        }
    })?;

    cmd_module.set("run_async", cmd_run_async)?;
    cmd_module.set("kill", kill)?;
    Ok(cmd_module)
}
