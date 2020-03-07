use rlua::prelude::LuaResult;

pub fn guard_io<R>(result: std::io::Result<R>) -> LuaResult<R> {
    result.map_err(|e| rlua::Error::ExternalError(std::sync::Arc::new(e)))
}
