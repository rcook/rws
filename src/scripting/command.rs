use crate::config::{ConfigHash, ConfigObject};
use crate::error::{user_error, user_error_result, Result};
use crate::scripting::lua;
use crate::scripting::CommandResult;

pub struct Command {
    language: String,
    use_prelude: bool,
    script: String,
    variables: Vec<(String, ConfigObject)>,
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
    pub fn new(root_hash: &ConfigHash, command_hash: &ConfigHash) -> Result<Command> {
        let language_default = root_hash
            .get("default-language")
            .and_then(|x| x.into_string())
            .unwrap_or(String::from("lua"));
        let language = command_hash
            .get("language")
            .and_then(|x| x.into_string())
            .unwrap_or(language_default)
            .to_string();

        let mut variables = Vec::new();
        if let Some(h) = root_hash.get("variables").and_then(|x| x.into_hash()) {
            for k in h.keys() {
                let obj = h.get(&k).expect("Unreachable");
                variables.push((k, obj));
            }
        }

        let language_hash_key = format!("{}-config", language);
        let language_hash_opt = root_hash
            .get(&language_hash_key)
            .and_then(|x| x.into_hash());

        let (preamble, use_prelude) = match language_hash_opt {
            Some(language_hash) => {
                let preamble = language_hash
                    .get("preamble")
                    .and_then(|x| x.into_string())
                    .unwrap_or(String::from(""));
                let use_prelude_default = language_hash
                    .get("use-prelude")
                    .and_then(|x| x.into_bool())
                    .unwrap_or(true);
                let use_prelude = command_hash
                    .get("use-prelude")
                    .and_then(|x| x.into_bool())
                    .unwrap_or(use_prelude_default);
                (preamble, use_prelude)
            }
            None => {
                let use_prelude = command_hash
                    .get("use-prelude")
                    .and_then(|x| x.into_bool())
                    .unwrap_or(true);
                (String::from(""), use_prelude)
            }
        };

        let script = command_hash
            .get("script")
            .and_then(|x| x.into_string())
            .ok_or_else(|| user_error("\"dependency-command\" missing required \"script\" field in workspace configuration"))?;

        let full_script = format!("{}\n\n{}", preamble, script);

        Ok(Command {
            language: language,
            use_prelude: use_prelude,
            script: full_script,
            variables: variables,
        })
    }

    pub fn eval0(&self) -> Result<Box<dyn CommandResult>> {
        match self.language.as_str() {
            "lua" => {
                let x = lua::eval(&self.script, self.use_prelude, &self.variables)?;
                Ok(Box::new(LuaResult::new(x)))
            }
            x => user_error_result(format!("Unsupported language \"{}\"", x)),
        }
    }

    pub fn eval1(&self) -> Result<()> {
        match self.language.as_str() {
            "lua" => lua::eval(&self.script, self.use_prelude, &self.variables),
            x => user_error_result(format!("Unsupported language \"{}\"", x)),
        }
    }
}
