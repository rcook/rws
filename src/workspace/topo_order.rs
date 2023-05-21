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
use anyhow::Result;
use std::hash::Hash;
use std::ops::Deref;
use topological_sort::TopologicalSort;

pub fn compute_topo_order<T, F>(targets: &[T], get_precs: F) -> Result<Vec<T>>
where
    T: Clone + Deref + Eq + Hash + Ord,
    F: Fn(&<T as Deref>::Target) -> Result<Vec<T>>,
{
    let mut all_precs = targets
        .iter()
        .map(|t| get_precs(t).map(|precs| (t.clone(), precs)))
        .collect::<Result<Vec<_>>>()?;

    let mut topo = TopologicalSort::new();
    while let Some((target, precs)) = all_precs.pop() {
        _ = topo.insert(target.clone());
        for prec in precs {
            topo.add_dependency(prec, target.clone());
        }
    }

    let mut order = Vec::new();
    while !topo.is_empty() {
        let mut v = topo.pop_all();
        v.sort();
        order.extend(v);
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::compute_topo_order;
    use anyhow::Result;
    use std::path::{Path, PathBuf};

    #[allow(clippy::unnecessary_wraps)]
    fn get_path_precs(target: &Path) -> Result<Vec<PathBuf>> {
        if target == Path::new("hello_world") {
            return Ok(vec![
                PathBuf::from("hello_world.o"),
                PathBuf::from("hello_world.c"),
                PathBuf::from("glibc.so"),
                PathBuf::from("zzz"),
            ]);
        }

        if target == Path::new("hello_world.o") {
            return Ok(vec![PathBuf::from("stdio.h")]);
        }

        Ok(vec![])
    }

    #[allow(clippy::unnecessary_wraps)]
    fn get_str_precs(target: &str) -> Result<Vec<String>> {
        if target == "hello_world" {
            return Ok(vec![
                String::from("hello_world.o"),
                String::from("hello_world.c"),
                String::from("glibc.so"),
                String::from("zzz"),
            ]);
        }

        if target == "hello_world.o" {
            return Ok(vec![String::from("stdio.h")]);
        }

        Ok(vec![])
    }

    #[test]
    fn test_paths() -> Result<()> {
        let targets = vec![
            PathBuf::from("glibc.so"),
            PathBuf::from("hello_world"),
            PathBuf::from("hello_world.c"),
            PathBuf::from("hello_world.o"),
            PathBuf::from("stdio.h"),
        ];

        let order = compute_topo_order(&targets, get_path_precs)?;
        assert_eq!(
            vec![
                Path::new("glibc.so"),
                Path::new("hello_world.c"),
                Path::new("stdio.h"),
                Path::new("zzz"),
                Path::new("hello_world.o"),
                Path::new("hello_world"),
            ],
            order
        );
        Ok(())
    }

    #[test]
    fn test_strs() -> Result<()> {
        let targets = vec![
            String::from("glibc.so"),
            String::from("hello_world"),
            String::from("hello_world.c"),
            String::from("hello_world.o"),
            String::from("stdio.h"),
        ];

        let order = compute_topo_order(&targets, get_str_precs)?;
        assert_eq!(
            vec![
                "glibc.so",
                "hello_world.c",
                "stdio.h",
                "zzz",
                "hello_world.o",
                "hello_world",
            ],
            order
        );
        Ok(())
    }
}
