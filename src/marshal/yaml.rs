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
use super::values::YamlValue;
use anyhow::{anyhow, bail, Result};
use rlua::prelude::LuaValue::{self, *};
use rlua::prelude::{LuaContext, LuaNil, LuaTable};
use serde_yaml::{Mapping, Number, Value};
use std::string::String as StdString;

#[allow(unused)]
pub fn yaml_to_lua<'a>(ctx: &LuaContext<'a>, obj: &YamlValue) -> Result<LuaValue<'a>> {
    use serde_yaml::Value::*;

    Ok(match obj {
        Null => LuaNil,
        Bool(value) => LuaValue::Boolean(*value),
        Number(value) => {
            if value.is_f64() {
                LuaValue::Number(value.as_f64().expect("must be f64"))
            } else if value.is_i64() {
                LuaValue::Integer(value.as_i64().expect("must be i64"))
            } else if value.is_u64() {
                LuaValue::Integer(i64::try_from(value.as_u64().expect("must be u64"))?)
            } else {
                bail!("Cannot convert {} to Lua numeric value", value)
            }
        }
        String(value) => LuaValue::String(ctx.create_string(value)?),
        Sequence(values) => LuaValue::Table(
            ctx.create_sequence_from(
                values
                    .iter()
                    .map(|value| yaml_to_lua(ctx, value))
                    .collect::<Result<Vec<_>>>()?,
            )?,
        ),
        Mapping(mapping) => LuaValue::Table(
            ctx.create_table_from(
                mapping
                    .iter()
                    .map(|(k, v)| {
                        k.as_str()
                            .ok_or(anyhow!("Unsupported key type in YAML"))
                            .and_then(|k_str| {
                                ctx.create_string(k_str)
                                    .map_err(|e| anyhow!(e))
                                    .and_then(|key| {
                                        yaml_to_lua(ctx, v)
                                            .map(|value| (LuaValue::String(key), value))
                                    })
                            })
                    })
                    .collect::<Result<Vec<(LuaValue, LuaValue)>>>()?,
            )?,
        ),
        Tagged(_) => bail!("Cannot convert tagged YAML to Lua"),
    })
}

#[allow(unused)]
pub fn lua_to_yaml(value: LuaValue, sub: bool) -> Result<YamlValue> {
    Ok(match value {
        Nil => Value::Null,
        Boolean(value) => Value::Bool(value),
        LightUserData(_value) => match sub {
            true => Value::String(StdString::from("(LIGHT_USER_DATA)")),
            false => bail!("cannot convert LightUserData"),
        },
        Integer(value) => Value::Number(Number::from(value)),
        Number(value) => Value::Number(Number::from(value)),
        String(value) => Value::String(StdString::from(value.to_str()?)),
        Table(table) => lua_table_to_yaml(table, sub)?,
        Function(_value) => match sub {
            true => Value::String(StdString::from("(FUNCTION)")),
            false => bail!("cannot convert Function"),
        },
        Thread(_value) => match sub {
            true => Value::String(StdString::from("(THREAD)")),
            false => bail!("cannot convert Thread"),
        },
        UserData(_value) => match sub {
            true => Value::String(StdString::from("(USER_DATA)")),
            false => bail!("cannot convert UserData"),
        },
        Error(_value) => match sub {
            true => Value::String(StdString::from("(ERROR)")),
            false => bail!("cannot convert Error"),
        },
    })
}

fn lua_table_to_yaml(table: LuaTable, sub: bool) -> Result<YamlValue> {
    if table.raw_len() == 0 {
        let mut mapping = Mapping::new();
        for p in table.pairs::<StdString, LuaValue>() {
            let (key, value) = p?;
            mapping.insert(Value::String(key), lua_to_yaml(value, sub)?);
        }
        Ok(Value::Mapping(mapping))
    } else {
        let mut values = Vec::new();
        for entry in table.sequence_values::<LuaValue>() {
            let value = entry?;
            values.push(lua_to_yaml(value, sub)?);
        }
        Ok(Value::Sequence(values))
    }
}

#[cfg(test)]
mod tests {
    use super::super::values::YamlValue;
    use super::{lua_to_yaml, yaml_to_lua};
    use anyhow::Result;
    use rlua::prelude::Lua;
    use rstest::rstest;
    use serde_yaml::Value::*;
    use serde_yaml::{Mapping, Number};
    use std::string::String as StdString;

    #[rstest]
    #[case(Null)]
    #[case(Bool(true))]
    #[case(Bool(false))]
    #[case(Number(Number::from(123)))]
    #[case(Number(Number::from(123u64)))]
    #[case(Number(Number::from(1.23)))]
    #[case(String(StdString::from("HELLO")))]
    #[case(Sequence(vec![Bool(true), String(StdString::from("HELLO")), Number(Number::from(123))]))]
    #[case(Mapping(Mapping::new()))]
    #[case(serde_yaml::from_str("key0: 123\nkey1: HELLO\n").expect("must succeed"))]
    fn roundtrip(#[case] input: YamlValue) -> Result<()> {
        let output = Lua::new().context(|ctx| -> Result<YamlValue> {
            let lua = yaml_to_lua(&ctx, &input)?;
            let output = lua_to_yaml(lua, false)?;
            Ok(output)
        })?;
        assert_eq!(input, output);
        Ok(())
    }

    #[test]
    fn variable() -> Result<()> {
        let input = r#"aaa:
  - bbb
  - ccc
  - ddd:
      eee:
      fff: hello"#;
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

        let obj = serde_yaml::from_str::<YamlValue>(input)?;
        let output = Lua::new().context(|ctx| -> Result<StdString> {
            let lua = yaml_to_lua(&ctx, &obj)?;
            ctx.globals().set("INPUT", lua)?;
            Ok(ctx.load(inspect_script).eval()?)
        })?;
        assert_eq!(
            "{ [aaa] = { [1] = bbb, [2] = ccc, [3] = { [ddd] = { [fff] = hello } } } }",
            output
        );
        Ok(())
    }
}
