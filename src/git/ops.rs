use crate::error::Result;
use crate::util::bracket;

use super::submodule::SubmoduleURLRewriter;

use dirs::home_dir;
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository, SubmoduleUpdateOptions};
use joat_git_url::GitUrl;
use std::path::Path;

pub fn clone_recursive(git_url: &GitUrl, clone_dir: &Path, branch: &str) -> Result<Repository> {
    let mut builder = RepoBuilder::new();
    builder.fetch_options(default_fetch_options());
    builder.branch(branch);

    let repo = builder.clone(&git_url.to_string(), clone_dir)?;

    // Workaround for libgit2/GitLab issue: cannot reliably handle relative URLs for Git submodules
    // We temporarily rewrite the contents of the .gitmodules
    let submodules_path = clone_dir.join(".gitmodules");
    bracket(
        || match submodules_path.is_file() {
            true => Ok(Some(SubmoduleURLRewriter::new(&submodules_path, git_url)?)),
            false => Ok(None),
        },
        |rewriter| {
            if let Some(x) = rewriter {
                x.restore()
            }
        },
        |_| {
            for mut submodule in repo.submodules()? {
                let mut update_opts = SubmoduleUpdateOptions::new();
                update_opts.fetch(default_fetch_options());
                submodule.update(true, Some(&mut update_opts))?
            }
            Result::Ok(())
        },
    )?;

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
