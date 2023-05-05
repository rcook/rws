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
use super::super::object::Object;
use anyhow::{anyhow, bail, Result};
use rlua::prelude::LuaValue::{self, *};
use rlua::prelude::{LuaContext, LuaNil, LuaTable};
use serde_json::{Map, Number, Value};
use std::string::String as StdString;

#[allow(unused)]
pub fn object_to_lua<'a>(ctx: &LuaContext<'a>, obj: &Object) -> Result<LuaValue<'a>> {
    use serde_json::Value::*;

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
        Array(values) => LuaValue::Table(
            ctx.create_sequence_from(
                values
                    .iter()
                    .map(|value| object_to_lua(ctx, value))
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
                                object_to_lua(ctx, v).map(|value| (LuaValue::String(key), value))
                            })
                    })
                    .collect::<Result<Vec<(LuaValue, LuaValue)>>>()?,
            )?,
        ),
    })
}

pub fn lua_to_object(value: LuaValue, sub: bool) -> Result<Object> {
    Ok(match value {
        Nil => Value::Null,
        Boolean(value) => Value::Bool(value),
        LightUserData(_value) => match sub {
            true => Value::String(StdString::from("(LIGHT_USER_DATA)")),
            false => bail!("cannot convert LightUserData"),
        },
        Integer(value) => Value::Number(Number::from(value)),
        Number(value) => Value::Number(
            Number::from_f64(value)
                .ok_or(anyhow!("Cannot convert {} to JSON numeric value", value))?,
        ),
        String(value) => Value::String(StdString::from(value.to_str()?)),
        Table(table) => lua_table_to_object(table, sub)?,
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

fn lua_table_to_object(table: LuaTable, sub: bool) -> Result<Object> {
    if table.raw_len() == 0 {
        let mut map = Map::new();
        for p in table.pairs::<StdString, LuaValue>() {
            let (key, value) = p?;
            map.insert(key, lua_to_object(value, sub)?);
        }
        Ok(Value::Object(map))
    } else {
        let mut values = Vec::new();
        for entry in table.sequence_values::<LuaValue>() {
            let value = entry?;
            values.push(lua_to_object(value, sub)?);
        }
        Ok(Value::Array(values))
    }
}

#[cfg(test)]
mod tests {
    use super::{lua_to_object, object_to_lua};
    use crate::scripting::object::Object;
    use anyhow::Result;
    use rlua::Lua;
    use rstest::rstest;
    use serde_json::json;
    use serde_json::Value::*;
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
    fn roundtrip(#[case] input: Object) -> Result<()> {
        let output = Lua::new().context(|ctx| -> Result<Object> {
            let lua = object_to_lua(&ctx, &input)?;
            let output = lua_to_object(lua, false)?;
            Ok(output)
        })?;
        assert_eq!(input, output);
        Ok(())
    }
}
