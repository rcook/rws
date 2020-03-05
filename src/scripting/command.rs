use crate::config::ConfigHash;
use crate::scripting::javascript;
use crate::scripting::lua;
use crate::scripting::{CommandInterpreter, CommandResult};

pub struct Command {
    language: String,
    use_prelude: bool,
    script: String,
}

struct LuaResult {
    strs: Vec<String>,
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
    pub fn new(root_hash: &ConfigHash, command_hash: &ConfigHash) -> Command {
        let language_default = root_hash.as_str("default-language").unwrap_or("lua");
        let language = command_hash
            .as_str("language")
            .unwrap_or(language_default)
            .to_string();

        let language_hash_key = format!("{}-config", language);
        let language_hash_opt = root_hash.as_hash(&language_hash_key);

        let (preamble, use_prelude) = match language_hash_opt {
            Some(language_hash) => {
                let preamble = language_hash.as_str("preamble").unwrap_or("").to_string();
                let use_prelude_default = language_hash.as_bool("use-prelude").unwrap_or(true);
                let use_prelude = command_hash
                    .as_bool("use-prelude")
                    .unwrap_or(use_prelude_default);
                (preamble, use_prelude)
            }
            None => {
                let use_prelude = command_hash.as_bool("use-prelude").unwrap_or(true);
                ("".to_string(), use_prelude)
            }
        };

        let script = command_hash.as_str("script").expect("Script not specified");
        let full_script = format!("{}\n\n{}", preamble, script);

        Command {
            language: language,
            use_prelude: use_prelude,
            script: full_script,
        }
    }

    pub fn eval(&self) -> Box<dyn CommandResult> {
        match self.language.as_str() {
            "javascript" => Box::new(
                javascript::JavaScriptInterpreter::new()
                    .eval(&self.script)
                    .unwrap(),
            ),
            "lua" => Box::new(LuaResult::new(lua::eval(&self.script, self.use_prelude))),
            _ => panic!("Unsupported language"),
        }
    }
}
