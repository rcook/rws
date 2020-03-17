use crate::config::ConfigHash;
use crate::error::{user_error, user_error_result, Result};
use crate::scripting::lua;
use crate::scripting::CommandResult;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

const LUA_SPECIAL_CHARACTERS: &AsciiSet = &CONTROLS.add(b'"').add(b'\'').add(b'\\');

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
    pub fn new(root_hash: &ConfigHash, command_hash: &ConfigHash) -> Result<Command> {
        let language_default = root_hash.as_str("default-language").unwrap_or("lua");
        let language = command_hash
            .as_str("language")
            .unwrap_or(language_default)
            .to_string();

        let variables_opt = root_hash.as_hash("variables");
        let variables_source = match variables_opt {
            Some(v) => match v.as_pairs() {
                Some(pairs) => pairs
                    .iter()
                    .map(|p| {
                        format!(
                            "local {} = prelude.percent_decode(\"{}\")",
                            Self::encode_lua_identifier(&p.0),
                            Self::encode_lua_string_literal(&p.1)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                None => String::from(""),
            },
            None => String::from(""),
        };

        let language_hash_key = format!("{}-config", language);
        let language_hash_opt = root_hash.as_hash(&language_hash_key);

        let (preamble, use_prelude) = match language_hash_opt {
            Some(language_hash) => {
                let preamble = language_hash.as_str("preamble").unwrap_or("").to_string();
                let use_prelude_default = language_hash.as_bool("use-prelude").unwrap_or(true);
                let use_prelude = command_hash
                    .as_bool("use-prelude")
                    .unwrap_or(use_prelude_default);
                (variables_source + "\n" + &preamble, use_prelude)
            }
            None => {
                let use_prelude = command_hash.as_bool("use-prelude").unwrap_or(true);
                (variables_source, use_prelude)
            }
        };

        let script = command_hash.as_str("script").ok_or_else(|| user_error("\"dependency-command\" missing required \"script\" field in workspace configuration"))?;
        let full_script = format!("{}\n\n{}", preamble, script);

        Ok(Command {
            language: language,
            use_prelude: use_prelude,
            script: full_script,
        })
    }

    pub fn eval0(&self) -> Result<Box<dyn CommandResult>> {
        match self.language.as_str() {
            "lua" => {
                let x = lua::eval0(&self.script, self.use_prelude)?;
                Ok(Box::new(LuaResult::new(x)))
            }
            x => user_error_result(format!("Unsupported language \"{}\"", x)),
        }
    }

    pub fn eval1(&self) -> Result<()> {
        match self.language.as_str() {
            "lua" => lua::eval1(&self.script, self.use_prelude),
            x => user_error_result(format!("Unsupported language \"{}\"", x)),
        }
    }

    fn encode_lua_identifier(str: &str) -> String {
        // TBD: Encode identifier here!
        str.to_string()
    }

    fn encode_lua_string_literal(str: &str) -> String {
        utf8_percent_encode(str, LUA_SPECIAL_CHARACTERS).to_string()
    }
}
