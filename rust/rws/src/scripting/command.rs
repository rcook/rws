use crate::config::ConfigHash;
use crate::scripting::{ CommandInterpreter, CommandResult };
use crate::scripting::javascript;
use crate::scripting::lua;

pub struct Command {
    language: String,
    use_prelude: bool,
    script: String
}

struct LuaResult {
    strs: Vec<String>
}

impl LuaResult {
    fn new(strs: Vec<String>) -> LuaResult {
        LuaResult { strs: strs }
    }
}

impl CommandResult for LuaResult {
    fn as_str_vec(&self) -> Option<Vec<String>> {
        Some(self.strs.clone())
    }
}

impl Command {
    pub fn new(root_hash: &ConfigHash, hash: &ConfigHash) -> Command {
        let language = hash.as_str("language").expect("Language not specified").to_string();
        let language_config_key = format!("{}-config", language);
        let language_config = root_hash.as_hash(&language_config_key).unwrap();
        let use_prelude_global = language_config.as_bool("use-prelude").unwrap_or(true);
        let preamble = language_config.as_str("preamble");
        let script = hash.as_str("script").expect("Script not specified").to_string();
        let full_script = format!("{}\n\n{}", preamble.unwrap_or(""), script);
        let use_prelude = hash.as_bool("use-prelude").unwrap_or(use_prelude_global);
        Command { language: language, use_prelude: use_prelude, script: full_script }
    }

    pub fn eval(&self) -> Box<dyn CommandResult> {
        match self.language.as_str() {
            "javascript" => Box::new(javascript::JavaScriptInterpreter::new().eval(&self.script).unwrap()),
            "lua" => Box::new(LuaResult::new(lua::eval(&self.script, self.use_prelude))),
            _ => panic!("Unsupported language")
        }
    }
}
