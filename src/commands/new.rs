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
use crate::marshal::YamlValue;
use crate::workspace::{
    Command, Config, Language, LanguageConfig, Session, Variables, WORKSPACE_CONFIG_FILE_NAME,
};
use anyhow::{bail, Result};
use joatmon::safe_write_file;

pub fn do_new(session: &Session) -> Result<()> {
    if let Some(config_path) = &session.config_path {
        bail!("A workspace already exists at {}", config_path.display())
    }

    let config_path = session.workspace_dir.join(&*WORKSPACE_CONFIG_FILE_NAME);
    if config_path.exists() {
        bail!(
            "A workspace configuration file already exists at {}",
            config_path.display()
        )
    }

    let config = Config {
        default_language: Some(Language::Lua),
        dependency_source: None,
        excluded_projects: None,
        init_command: Some(Command {
            language: None,
            use_prelude: None,
            script: String::from("print(\"init_command\")"),
        }),
        lua_config: Some(LanguageConfig {
            preamble: Some(String::from("print(\"preamble\")")),
            use_prelude: Some(true),
        }),
        variables: Some(Variables::from([
            (
                String::from("VARIABLE0"),
                YamlValue::String(String::from("VALUE0")),
            ),
            (
                String::from("VARIABLE1"),
                YamlValue::String(String::from("VALUE1")),
            ),
        ])),
    };

    let yaml_str = serde_yaml::to_string(&config)?;
    safe_write_file(&config_path, yaml_str, false)?;
    Ok(())
}
