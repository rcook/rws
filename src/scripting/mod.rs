pub mod command;

mod helpers;
mod lua;
mod prelude;
mod xml;

pub trait CommandResult {
    fn as_str_vec(&self) -> Option<Vec<String>>;
}

pub trait CommandInterpreter<T: CommandResult> {
    fn eval(&mut self, src: &str) -> Option<T>;
}
