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
use anyhow::{anyhow, Result};
use serde_json::value::Index;
use serde_json::Value;
use std::fmt::Display;

pub trait ObjectTrait {
    fn get_required_bool<I>(&self, index: I) -> Result<bool>
    where
        I: Index + Display;
    fn get_required_str<I>(&self, index: I) -> Result<&str>
    where
        I: Index + Display;
}

pub type Object = Value;

impl ObjectTrait for Object {
    fn get_required_bool<I>(&self, index: I) -> Result<bool>
    where
        I: Index + Display,
    {
        self.get(&index)
            .ok_or(anyhow!("Required field {} missing", index))?
            .as_bool()
            .ok_or(anyhow!("Required field {} is not of expected type", index))
    }

    fn get_required_str<I>(&self, index: I) -> Result<&str>
    where
        I: Index + Display,
    {
        self.get(&index)
            .ok_or(anyhow!("Required field {} missing", index))?
            .as_str()
            .ok_or(anyhow!("Required field {} is not of expected type", index))
    }
}
