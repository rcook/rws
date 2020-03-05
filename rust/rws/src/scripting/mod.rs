pub mod command;
mod javascript;
mod lua;
mod lua_rws_prelude;

pub trait CommandResult {
    fn as_str_vec(&self) -> Option<Vec<String>>;
}

pub trait CommandInterpreter<T: CommandResult> {
    fn eval(&mut self, src: &str) -> Option<T>;
}
