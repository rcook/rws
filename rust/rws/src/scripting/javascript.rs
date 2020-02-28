use super::{ CommandInterpreter, CommandResult };

use boa::builtins::function::NativeFunctionData;
use boa::builtins::object::ObjectKind;
use boa::builtins::value::{ ResultValue, Value, ValueData, to_value };
use boa::exec::{ Executor, Interpreter };
use boa::forward_val;
use boa::realm::Realm;
use gc::Gc;

// Copied from Boa package
macro_rules! make_builtin_fn {
    ($fn:ident, named $name:expr, with length $l:tt, of $p:ident) => {
        let $fn = to_value($fn as NativeFunctionData);
        $fn.set_field_slice("length", to_value($l));
        $p.set_field_slice($name, $fn);
    };
    ($fn:ident, named $name:expr, of $p:ident) => {
        make_builtin_fn!($fn, named $name, with length 0, of $p);
    };
}

fn read_file(_: &Value, _args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Gc::new(ValueData::String("(read_file)".to_string())))
}

fn read_file_lines(_: &Value, _args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Gc::new(ValueData::String("(read_file_lines)".to_string())))
}

pub struct JavaScriptResult {
    value: Value
}

impl JavaScriptResult {
    fn new(value: Value) -> JavaScriptResult {
        JavaScriptResult { value: value }
    }

    fn as_i32(value: &ValueData) -> Option<i32> {
        match value {
            ValueData::Integer(x) => Some(*x),
            _ => None
        }
    }

    fn as_string(value: &ValueData) -> Option<String> {
        match value {
            ValueData::String(ref x) => Some(x.clone()),
            _ => None
        }
    }
}

impl CommandResult for JavaScriptResult {
    fn as_str_vec(&self) -> Option<Vec<String>> {
        let obj = match *self.value {
            ValueData::Object(ref x) => Some(x),
            _ => None
        }?;

        if obj.borrow().kind != ObjectKind::Array {
            return None
        }

        (0..JavaScriptResult::as_i32(&self.value.get_field_slice("length"))?)
            .map(|idx| JavaScriptResult::as_string(&self.value.get_field_slice(&idx.to_string())))
            .collect()
    }
}

pub struct JavaScriptInterpreter {
    engine: Interpreter
}

impl JavaScriptInterpreter {
    pub fn new() -> JavaScriptInterpreter {
        let realm = Realm::create();
        realm.global_obj.set_field_slice("rws", Gc::new(ValueData::Integer(12345)));

        let prelude = ValueData::new_obj(Some(&realm.global_obj));
        make_builtin_fn!(read_file, named "readFile", with length 1, of prelude);
        make_builtin_fn!(read_file_lines, named "readFileLines", with length 1, of prelude);

        realm.global_obj.set_field_slice("prelude", prelude);

        let engine: Interpreter = Executor::new(realm);
        JavaScriptInterpreter { engine: engine }
    }
}

impl CommandInterpreter<JavaScriptResult> for JavaScriptInterpreter {
    fn eval(&mut self, src: &str) -> Option<JavaScriptResult> {
        Some(JavaScriptResult::new(forward_val(&mut self.engine, src).ok()?))
    }
}
