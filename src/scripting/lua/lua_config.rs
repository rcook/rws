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
use crate::config::ConfigObject;
use anyhow::Result;
use rlua::{Context, Value};
use yaml_rust::yaml::Yaml;

pub fn translate_config_to_lua<'a>(
    lua_ctx: Context<'a>,
    config_object: &ConfigObject,
) -> Result<Value<'a>> {
    Ok(translate_helper(lua_ctx, &config_object.yaml)?)
}

fn translate_helper<'a>(lua_ctx: Context<'a>, yaml: &Yaml) -> rlua::Result<Value<'a>> {
    match yaml {
        Yaml::String(value) => lua_ctx.create_string(&value).map(Value::String),
        Yaml::Array(value) => lua_ctx
            .create_sequence_from(
                value
                    .iter()
                    .map(|x| translate_helper(lua_ctx, x))
                    .collect::<rlua::Result<Vec<_>>>()?,
            )
            .map(Value::Table),
        Yaml::Hash(value) => lua_ctx
            .create_table_from(
                value
                    .iter()
                    .map(|(k, v)| {
                        k.as_str()
                            .ok_or_else(|| rlua::Error::RuntimeError(String::from("Invalid YAML")))
                            .and_then(|k_str| {
                                lua_ctx.create_string(k_str).and_then(|key| {
                                    translate_helper(lua_ctx, v)
                                        .map(|value| (Value::String(key), value))
                                })
                            })
                    })
                    .collect::<rlua::Result<Vec<(Value, Value)>>>()?,
            )
            .map(Value::Table),
        _ => unimplemented!("Unsupported YAML node type"),
    }
}

#[cfg(test)]
mod tests {
    use super::translate_helper;
    use anyhow::Result;
    use rlua::prelude::Lua;
    use yaml_rust::YamlLoader;

    fn render_yaml_with_lua(yaml_text: &str) -> Result<String> {
        let inspect_script = r#"
function inspect(obj)
    if type(obj) == "table" then
        local s = "{ "
        local idx = 0
        for key, value in pairs(obj) do
            if idx > 0 then
                s = s .. ", "
            end

            if not (type(k) ~= "number") then
                key = "\"" .. key .. "\""
            end

            s = s .. "[" .. key .. "] = " .. inspect(value)

            idx = idx + 1
        end
        return s .. " }"
    else
        return tostring(obj)
    end
end

return inspect(INPUT)
"#;

        let mut docs = YamlLoader::load_from_str(yaml_text)?;
        assert_eq!(1, docs.len());
        let yaml = docs.remove(0);
        Lua::new().context(|lua_ctx| -> Result<String> {
            let value = translate_helper(lua_ctx, &yaml)?;
            lua_ctx.globals().set("INPUT", value)?;
            Ok(lua_ctx.load(inspect_script).eval()?)
        })
    }

    #[test]
    fn translate_helper_basics() -> Result<()> {
        assert_eq!(
            "{ [1] = one, [2] = two }",
            render_yaml_with_lua(
                r#"- one
- two
"#
            )?
        );
        Ok(())
    }
}
