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
#[allow(unused)]
pub fn to_lua<'a>(
    ctx: &rlua::Context<'a>,
    value: &serde_json::Value,
) -> anyhow::Result<rlua::Value<'a>> {
    use anyhow::{anyhow, bail, Result};
    use rlua::prelude::{LuaNil, LuaValue};
    use serde_json::Value::*;

    Ok(match value {
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
        Array(values) => LuaValue::Table(
            ctx.create_sequence_from(
                values
                    .iter()
                    .map(|value| to_lua(ctx, value))
                    .collect::<Result<Vec<_>>>()?,
            )?,
        ),
        Object(map) => LuaValue::Table(
            ctx.create_table_from(
                map.iter()
                    .map(|(k, v)| {
                        ctx.create_string(k)
                            .map_err(|e| anyhow!(e))
                            .and_then(|key| {
                                to_lua(ctx, v).map(|value| (LuaValue::String(key), value))
                            })
                    })
                    .collect::<Result<Vec<(LuaValue, LuaValue)>>>()?,
            )?,
        ),
    })
}

#[allow(unused)]
pub fn from_lua(value: rlua::prelude::LuaValue) -> anyhow::Result<serde_json::Value> {
    use anyhow::{anyhow, bail};
    use rlua::prelude::LuaValue::{self, *};
    use serde_json::{Map, Number, Value};
    use std::string::String as StdString;

    Ok(match value {
        Nil => Value::Null,
        Boolean(value) => Value::Bool(value),
        LightUserData(_value) => bail!("cannot convert LightUserData"),
        Integer(value) => Value::Number(Number::from(value)),
        Number(value) => Value::Number(
            Number::from_f64(value)
                .ok_or(anyhow!("Cannot convert {} to JSON numeric value", value))?,
        ),
        String(value) => Value::String(StdString::from(value.to_str()?)),
        Table(table) => {
            let count = table.raw_len();
            if count == 0 {
                let mut map = Map::new();
                for p in table.pairs::<StdString, LuaValue>() {
                    let (key, value) = p?;
                    map.insert(key, from_lua(value)?);
                }
                Value::Object(map)
            } else {
                let mut values = Vec::new();
                for entry in table.sequence_values::<LuaValue>() {
                    let value = entry?;
                    values.push(from_lua(value)?);
                }
                Value::Array(values)
            }
        }
        Function(_value) => bail!("cannot convert Function"),
        Thread(_value) => bail!("cannot convert Thread"),
        UserData(_value) => bail!("cannot convert UserData"),
        Error(_value) => bail!("cannot convert Error"),
    })
}

#[cfg(test)]
mod tests {
    use super::{from_lua, to_lua};
    use anyhow::Result;
    use rlua::Lua;
    use rstest::rstest;
    use serde_json::json;
    use serde_json::Value::{self, *};
    use serde_json::{Map, Number};
    use std::string::String as StdString;

    #[rstest]
    #[case(Null)]
    #[case(Bool(true))]
    #[case(Bool(false))]
    #[case(Number(Number::from(123)))]
    #[case(Number(Number::from(123u64)))]
    #[case(Number(Number::from_f64(1.23).expect("must succeed")))]
    #[case(String(StdString::from("HELLO")))]
    #[case(Array(vec![Bool(true), String(StdString::from("HELLO")), Number(Number::from(123))]))]
    #[case(Object(Map::new()))]
    #[case(json![{"key0": 123, "key1": "HELLO"}])]
    fn roundtrip(#[case] input_json: Value) -> Result<()> {
        let output_json = Lua::new().context(|ctx| -> Result<Value> {
            let lua = to_lua(&ctx, &input_json)?;
            let output_json = from_lua(lua)?;
            Ok(output_json)
        })?;
        assert_eq!(input_json, output_json);
        Ok(())
    }

    /*
    fn to_json_pretty(value: &Value) -> Result<String> {
        Ok(serde_json::to_string_pretty(value)?)
    }

     #[test]
    fn basics() -> Result<()> {
        let value = json!({
            "code": 200,
            "success": true,
            "payload": {
                "features": [
                    "serde",
                    "json"
                ]
            }
        });
        println!("value={}", to_json_pretty(&value)?);
        Ok(())
    }
    */
}
