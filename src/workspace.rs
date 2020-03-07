use crate::config::{Config, ConfigHash};
use crate::deps::get_deps;
use crate::error::{user_error, Result};
use crate::scripting::command::Command;

use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use topological_sort::TopologicalSort;

const WORKSPACE_CONFIG_FILE_NAME: &str = "rws-workspace.yaml";

pub struct Workspace {
    pub config_path: Option<PathBuf>,
    pub root_dir: PathBuf,
    pub project_dirs_alpha: Vec<PathBuf>,
    pub project_dirs_topo: Vec<PathBuf>,
}

impl Workspace {
    pub fn find(search_dir: &Path) -> Result<Workspace> {
        let result = Self::find_impl(search_dir)?;
        Ok(result.unwrap_or_else(|| {
            let project_dirs_alpha = vec![search_dir.to_path_buf()];
            let project_dirs_topo = vec![search_dir.to_path_buf()];
            Workspace {
                config_path: None,
                root_dir: search_dir.to_path_buf(),
                project_dirs_alpha: project_dirs_alpha,
                project_dirs_topo: project_dirs_topo,
            }
        }))
    }

    fn find_impl(search_dir: &Path) -> Result<Option<Workspace>> {
        let x = search_dir.join(WORKSPACE_CONFIG_FILE_NAME);
        if x.exists() {
            let workspace = Self::traverse(x, search_dir.to_path_buf())?;
            return Ok(Some(workspace));
        }

        match search_dir.parent() {
            Some(p) => Self::find_impl(p),
            None => Ok(None),
        }
    }

    fn traverse(config_path: PathBuf, root_dir: PathBuf) -> Result<Workspace> {
        let config = Config::read_config_file(&config_path)?;
        let root_hash = config
            .as_hash()
            .ok_or_else(|| user_error("Invalid config hash"))?;
        let excluded_project_dirs = root_hash
            .as_str_vec("excluded-projects")
            .unwrap_or_else(|| Vec::new())
            .into_iter()
            .map(|x| root_dir.join(x))
            .collect::<HashSet<_>>();
        let dependencies_hash_opt = root_hash.as_hash("dependencies");
        let dependency_command_hash_opt = root_hash.as_hash("dependency-command");
        match (&dependencies_hash_opt, &dependency_command_hash_opt) {
            (Some(_), Some(_)) => {
                panic!("Invalid: cannot specify both dependencies and dependency-command")
            }
            (Some(dependencies_hash), None) => Self::traverse_with_dependencies(
                config_path,
                root_dir,
                &excluded_project_dirs,
                &dependencies_hash,
            ),
            (None, Some(dependency_command_hash)) => Self::traverse_with_dependency_command(
                config_path,
                root_dir,
                &excluded_project_dirs,
                &root_hash,
                &dependency_command_hash,
            ),
            (None, None) => {
                panic!("Invalid: must specify one of dependencies or dependency-command")
            }
        }
    }

    fn traverse_with_dependencies(
        config_path: PathBuf,
        root_dir: PathBuf,
        excluded_project_dirs: &HashSet<PathBuf>,
        dependency_command_hash: &ConfigHash,
    ) -> Result<Workspace> {
        Self::traverse_helper(
            config_path,
            &root_dir,
            excluded_project_dirs,
            |project_dir| {
                let project_name = match project_dir.components().last().unwrap() {
                    Component::Normal(s) => s,
                    _ => panic!("Unimplemented"),
                };
                match dependency_command_hash.as_vec(project_name.to_str().unwrap()) {
                    Some(v) => Ok((0..v.len())
                        .into_iter()
                        .map(|i| {
                            String::from(root_dir.join(v.as_str(i).unwrap()).to_str().unwrap())
                        })
                        .collect()),
                    None => Ok(Vec::new()),
                }
            },
        )
    }

    fn traverse_with_dependency_command(
        config_path: PathBuf,
        root_dir: PathBuf,
        excluded_project_dirs: &HashSet<PathBuf>,
        root_hash: &ConfigHash,
        dependency_command_hash: &ConfigHash,
    ) -> Result<Workspace> {
        let dependency_command = Command::new(root_hash, dependency_command_hash);
        Self::traverse_helper(
            config_path,
            &root_dir,
            excluded_project_dirs,
            |project_dir| {
                get_deps(&project_dir, &dependency_command).map(|x| {
                    x.into_iter()
                        .map(|x| String::from(root_dir.join(x).to_str().unwrap()))
                        .collect::<Vec<_>>()
                })
            },
        )
    }

    fn get_project_dirs_alpha(
        root_dir: &Path,
        excluded_project_dirs: &HashSet<PathBuf>,
    ) -> std::io::Result<Vec<PathBuf>> {
        let mut project_dirs_alpha = Vec::new();
        for entry in fs::read_dir(&root_dir)? {
            let e = entry?;
            let project_dir = e.path();
            if !excluded_project_dirs.contains(&project_dir) && project_dir.is_dir() {
                let git_dir = project_dir.join(".git");
                if git_dir.is_dir() {
                    project_dirs_alpha.push(project_dir)
                }
            }
        }

        Ok(project_dirs_alpha)
    }

    fn topo_sort_project_dirs<F>(project_dirs_alpha: &Vec<PathBuf>, f: F) -> Result<Vec<PathBuf>>
    where
        F: Fn(&Path) -> Result<Vec<String>>,
    {
        let mut ts = TopologicalSort::<String>::new();
        for project_dir in project_dirs_alpha {
            let deps = f(project_dir)?;

            // TBD: Figure out how to store PathBuf/Path directly in TopologicalSort
            for dep in &deps {
                let p = Path::new(dep);
                if !p.is_dir() {
                    // TBD: Do not fail with panic!
                    panic!(format!(
                        "Project directory {} does not exist",
                        p.to_str().unwrap()
                    ))
                }
            }

            // TBD: Don't convert to string etc.
            ts.insert(String::from(project_dir.to_str().unwrap()));

            for dep in &deps {
                // TBD: Don't convert to string etc.
                ts.add_dependency(dep, project_dir.to_str().unwrap())
            }
        }

        let mut project_dirs_topo = Vec::new();
        while !ts.is_empty() {
            let mut v = ts.pop_all();
            v.sort();
            for p in v {
                project_dirs_topo.push(PathBuf::from(&p))
            }
        }

        Ok(project_dirs_topo)
    }

    fn traverse_helper<F>(
        config_path: PathBuf,
        root_dir: &PathBuf,
        excluded_project_dirs: &HashSet<PathBuf>,
        f: F,
    ) -> Result<Workspace>
    where
        F: Fn(&Path) -> Result<Vec<String>>,
    {
        let project_dirs_alpha =
            Self::get_project_dirs_alpha(&root_dir, &excluded_project_dirs).unwrap();
        let project_dirs_topo = Self::topo_sort_project_dirs(&project_dirs_alpha, f)?;

        Ok(Workspace {
            config_path: Some(config_path),
            root_dir: root_dir.to_path_buf(),
            project_dirs_alpha: project_dirs_alpha,
            project_dirs_topo: project_dirs_topo,
        })
    }
}
