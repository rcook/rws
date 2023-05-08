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
use super::traits::Eval;
use crate::config::{Command, Language, Variables};
use crate::workspace::Session;
use anyhow::Result;
use std::fmt::Debug;

const DEFAULT_LANGUAGE: Language = Language::Lua;
const DEFAULT_PREAMBLE: &str = "";
const DEFAULT_USE_PRELUDE: bool = true;

fn default_variables() -> Variables {
    Variables::new()
}

pub fn eval_script_command<T>(session: &Session, command: &Command) -> Result<T>
where
    T: Debug + Eval,
{
    let default_language = session
        .definition
        .as_ref()
        .and_then(|d| d.default_language.clone())
        .unwrap_or(DEFAULT_LANGUAGE);

    let language = command
        .language
        .as_ref()
        .unwrap_or(&default_language)
        .clone();

    let language_config_opt = match &language {
        Language::Lua => session
            .definition
            .as_ref()
            .and_then(|d| d.lua_config.clone()),
    };

    let (preamble, use_prelude) = match language_config_opt {
        Some(language_config) => {
            let preamble = language_config
                .preamble
                .unwrap_or(String::from(DEFAULT_PREAMBLE));
            let default_use_prelude = language_config.use_prelude.unwrap_or(DEFAULT_USE_PRELUDE);
            let use_prelude = command.use_prelude.unwrap_or(default_use_prelude);
            (preamble, use_prelude)
        }
        None => {
            let use_prelude = command.use_prelude.unwrap_or(DEFAULT_USE_PRELUDE);
            (String::from(DEFAULT_PREAMBLE), use_prelude)
        }
    };

    let script = command.script.clone();

    let variables = session
        .definition
        .as_ref()
        .and_then(|d| d.variables.clone())
        .unwrap_or(default_variables());

    match language {
        Language::Lua => super::lua::eval(session, &preamble, &script, use_prelude, &variables),
    }
}
