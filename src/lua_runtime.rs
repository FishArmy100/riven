use mlua::{Lua, Result};


pub fn run_lua(text: &str) -> Result<()>
{
    let lua = Lua::new();
    lua.load(text).exec()?;
    Ok(())
}