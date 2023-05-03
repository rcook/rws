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
use super::submodule::SubmoduleURLRewriter;
use anyhow::Result;
use dirs::home_dir;
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository, SubmoduleUpdateOptions};
use joat_git_url::GitUrl;
use std::path::Path;

struct SubmoduleTracker {
    rewriter: Option<SubmoduleURLRewriter>,
}

impl SubmoduleTracker {
    fn empty() -> Self {
        Self { rewriter: None }
    }

    fn new(submodules_path: &Path, remote_git_url: &GitUrl) -> Result<Self> {
        Ok(Self {
            rewriter: Some(SubmoduleURLRewriter::new(submodules_path, remote_git_url)?),
        })
    }
}

impl Drop for SubmoduleTracker {
    fn drop(&mut self) {
        match &self.rewriter {
            Some(x) => {
                x.restore();
                self.rewriter = None;
            }
            None => {}
        }
    }
}

pub fn clone_recursive(git_url: &GitUrl, clone_dir: &Path, branch: &str) -> Result<Repository> {
    let mut builder = RepoBuilder::new();
    builder.fetch_options(default_fetch_options());
    builder.branch(branch);

    let repo = builder.clone(&git_url.to_string(), clone_dir)?;

    // Workaround for libgit2/GitLab issue: cannot reliably handle relative URLs for Git submodules
    // We temporarily rewrite the contents of the .gitmodules
    let submodules_path = clone_dir.join(".gitmodules");

    let tracker = match submodules_path.is_file() {
        true => SubmoduleTracker::new(&submodules_path, git_url)?,
        false => SubmoduleTracker::empty(),
    };

    for mut submodule in repo.submodules()? {
        let mut update_opts = SubmoduleUpdateOptions::new();
        update_opts.fetch(default_fetch_options());
        submodule.update(true, Some(&mut update_opts))?
    }

    drop(tracker);

    Ok(repo)
}

fn default_fetch_options<'a>() -> FetchOptions<'a> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, user_name_from_url, _allowed_types| {
        let dir = home_dir()
            .ok_or_else(|| git2::Error::from_str("Could not determine home directory"))?;
        let id_rsa_path = dir.join(".ssh").join("id_rsa");
        Cred::ssh_key(
            user_name_from_url.expect("user_name_from_url was None"),
            None,
            &id_rsa_path,
            None,
        )
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    fetch_opts
}
