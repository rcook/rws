pub mod command;
mod command_prelude;
mod lua;

pub trait CommandResult {
    fn as_str_vec(&self) -> Option<Vec<String>>;
}

pub trait CommandInterpreter<T: CommandResult> {
    fn eval(&mut self, src: &str) -> Option<T>;
}
