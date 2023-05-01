// The MIT License (MIT)
//
// Copyright (c) 2020-3 Richard Cook
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
use super::variables::Variables;
use super::{javascript, lua};
use crate::config::ConfigHash;
use crate::workspace::Workspace;
use anyhow::{anyhow, bail, Result};

mod config_value {
    pub const JAVASCRIPT: &str = "javascript";
    pub const LUA: &str = "lua";
}

pub trait Evaluatable: lua::Evaluatable + javascript::Evaluatable {}

impl<T: lua::Evaluatable + javascript::Evaluatable> Evaluatable for T {}

#[derive(Debug)]
pub struct ScriptCommand {
    language: String,
    use_prelude: bool,
    preamble: String,
    script: String,
    variables: Variables,
}

impl ScriptCommand {
    pub fn new(root_hash: &ConfigHash, command_hash: &ConfigHash) -> Result<Self> {
        use crate::config_key::*;
        use config_value::*;

        let language_default = root_hash
            .get(DEFAULT_LANGUAGE)
            .and_then(|x| x.into_string())
            .unwrap_or_else(|| String::from(LUA));
        let language = command_hash
            .get(LANGUAGE)
            .and_then(|x| x.into_string())
            .unwrap_or(language_default);

        let variables = Self::get_variables(root_hash)?;

        let language_hash_key = format!("{}-config", language);
        let language_hash_opt = root_hash
            .get(&language_hash_key)
            .and_then(|x| x.into_hash());

        let (preamble, use_prelude) = match language_hash_opt {
            Some(language_hash) => {
                let preamble = language_hash
                    .get(PREAMBLE)
                    .and_then(|x| x.into_string())
                    .unwrap_or_else(|| String::from(""));
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
            .ok_or_else(|| {
                anyhow!(
                    "\"{}\" element missing required \"{}\" field in workspace configuration",
                    DEPENDENCY_COMMAND,
                    SCRIPT
                )
            })?;

        Ok(ScriptCommand {
            language,
            use_prelude,
            preamble,
            script,
            variables,
        })
    }

    pub fn eval<T: Evaluatable>(&self, workspace: &Workspace) -> Result<T> {
        match self.language.as_str() {
            config_value::JAVASCRIPT => javascript::eval(
                workspace,
                &self.preamble,
                &self.script,
                self.use_prelude,
                &self.variables,
            ),
            config_value::LUA => lua::eval(
                workspace,
                &self.preamble,
                &self.script,
                self.use_prelude,
                &self.variables,
            ),
            language => bail!("Unsupported language \"{}\"", language),
        }
    }

    fn get_variables(root_hash: &ConfigHash) -> Result<Variables> {
        use crate::config_key::*;

        let mut values = Vec::new();
        if let Some(h) = root_hash.get(VARIABLES).and_then(|x| x.into_hash()) {
            let keys = h.keys().ok_or_else(|| {
                anyhow!("Invalid keys in \"{}\" configuration element", VARIABLES)
            })?;
            for k in keys {
                let obj = h.get(&k).expect("Unreachable");
                values.push((k, obj));
            }
        };
        Ok(Variables::new(values))
    }
}
