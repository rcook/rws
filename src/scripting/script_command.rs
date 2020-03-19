use crate::config::{ConfigHash, ConfigObject};
use crate::error::{user_error, user_error_result, Result};
use crate::scripting::lua;

use rlua::FromLuaMulti;

mod config_value {
    pub const LUA: &str = "lua";
}

pub struct ScriptCommand {
    language: String,
    use_prelude: bool,
    script: String,
    variables: Vec<(String, ConfigObject)>,
}

impl ScriptCommand {
    pub fn new(root_hash: &ConfigHash, command_hash: &ConfigHash) -> Result<Self> {
        use crate::config_key::*;
        use config_value::*;

        let language_default = root_hash
            .get(DEFAULT_LANGUAGE)
            .and_then(|x| x.into_string())
            .unwrap_or(String::from(LUA));
        let language = command_hash
            .get(LANGUAGE)
            .and_then(|x| x.into_string())
            .unwrap_or(language_default)
            .to_string();

        let mut variables = Vec::new();
        if let Some(h) = root_hash.get(VARIABLES).and_then(|x| x.into_hash()) {
            let keys = h.keys().ok_or_else(|| {
                user_error(format!(
                    "Invalid keys in \"{}\" configuration element",
                    VARIABLES
                ))
            })?;
            for k in keys {
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
                    .get(PREAMBLE)
                    .and_then(|x| x.into_string())
                    .unwrap_or(String::from(""));
                let use_prelude_default = language_hash
                    .get(USE_PRELUDE)
                    .and_then(|x| x.into_bool())
                    .unwrap_or(true);
                let use_prelude = command_hash
                    .get(USE_PRELUDE)
                    .and_then(|x| x.into_bool())
                    .unwrap_or(use_prelude_default);
                (preamble, use_prelude)
            }
            None => {
                let use_prelude = command_hash
                    .get(USE_PRELUDE)
                    .and_then(|x| x.into_bool())
                    .unwrap_or(true);
                (String::from(""), use_prelude)
            }
        };

        let script = command_hash
            .get(SCRIPT)
            .and_then(|x| x.into_string())
            .ok_or_else(|| user_error(format!("\"dependency-command\" missing required \"{}\" field in workspace configuration", SCRIPT)))?;

        let full_script = format!("{}\n\n{}", preamble, script);

        Ok(ScriptCommand {
            language: language,
            use_prelude: use_prelude,
            script: full_script,
            variables: variables,
        })
    }

    // TBD: Figure out how to prevent `FromLuaMulti` from leaking!
    pub fn eval<R: for<'lua> FromLuaMulti<'lua>>(&self) -> Result<R> {
        match self.language.as_str() {
            "lua" => lua::eval(&self.script, self.use_prelude, &self.variables),
            language => user_error_result(format!("Unsupported language \"{}\"", language)),
        }
    }
}
