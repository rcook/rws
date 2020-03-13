use crate::config::{Config, ConfigHash};
use crate::deps::get_deps;
use crate::error::{user_error, user_error_result, Result};
use crate::os::{get_base_name, path_to_str};
use crate::scripting::command::Command;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use topological_sort::TopologicalSort;

const WORKSPACE_CONFIG_FILE_NAME: &str = "rws-workspace.yaml";

pub struct Workspace {
    pub root_dir: PathBuf,
    pub config_path: Option<PathBuf>,
    pub project_dirs_alpha: Vec<PathBuf>,
    pub project_dirs_topo: Option<Vec<PathBuf>>,
}

impl Workspace {
    pub fn find(search_dir: &Path) -> Result<Workspace> {
        let mut p = search_dir;
        loop {
            let config_path = p.join(WORKSPACE_CONFIG_FILE_NAME);
            if config_path.exists() {
                match Config::read_config_file(&config_path)? {
                    Some(config) => {
                        return Self::traverse_config(p.to_path_buf(), config_path, &config)
                    }
                    None => {
                        return Self::traverse_no_dependencies_or_dependency_command(
                            p.to_path_buf(),
                            Some(config_path),
                            &HashSet::new(),
                        )
                    }
                }
            }
            match p.parent() {
                Some(parent) => p = parent,
                None => {
                    return Self::traverse_no_dependencies_or_dependency_command(
                        search_dir.to_path_buf(),
                        None,
                        &HashSet::new(),
                    )
                }
            }
        }
    }

    fn traverse_config(
        root_dir: PathBuf,
        config_path: PathBuf,
        config: &Config,
    ) -> Result<Workspace> {
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
            (Some(_), Some(_)) => user_error_result("Must specify at most one of \"dependencies\" and \"dependency-command\" in workspace configuration"),
            (Some(dependencies_hash), None) => Self::traverse_with_dependencies(
                root_dir,
                config_path,
                &excluded_project_dirs,
                &dependencies_hash,
            ),
            (None, Some(dependency_command_hash)) => Self::traverse_with_dependency_command(
                root_dir,
                config_path,
                &excluded_project_dirs,
                &root_hash,
                &dependency_command_hash,
            ),
            (None, None) => Self::traverse_no_dependencies_or_dependency_command(root_dir, Some(config_path),  &excluded_project_dirs)
        }
    }

    fn traverse_no_dependencies_or_dependency_command(
        root_dir: PathBuf,
        config_path: Option<PathBuf>,
        excluded_project_dirs: &HashSet<PathBuf>,
    ) -> Result<Workspace> {
        Ok(Workspace {
            root_dir: root_dir.to_path_buf(),
            config_path: config_path,
            project_dirs_alpha: Self::get_project_dirs_alpha(&root_dir, &excluded_project_dirs)?,
            project_dirs_topo: None,
        })
    }

    fn traverse_with_dependencies(
        root_dir: PathBuf,
        config_path: PathBuf,
        excluded_project_dirs: &HashSet<PathBuf>,
        dependency_command_hash: &ConfigHash,
    ) -> Result<Workspace> {
        Self::traverse_helper(
            &root_dir,
            Some(config_path),
            excluded_project_dirs,
            |project_dir| {
                let project_name = get_base_name(project_dir);
                match dependency_command_hash.as_vec(project_name) {
                    Some(v) => (0..v.len())
                        .into_iter()
                        .map(|i| {
                            v.as_str(i)
                                .ok_or_else(|| {
                                    user_error(format!(
                                        "Invalid dependency {} for project {}",
                                        v.as_display(i),
                                        project_name
                                    ))
                                })
                                .map(|s| String::from(path_to_str(&root_dir.join(s))))
                        })
                        .collect(),
                    None => Ok(Vec::new()),
                }
            },
        )
    }

    fn traverse_with_dependency_command(
        root_dir: PathBuf,
        config_path: PathBuf,
        excluded_project_dirs: &HashSet<PathBuf>,
        root_hash: &ConfigHash,
        dependency_command_hash: &ConfigHash,
    ) -> Result<Workspace> {
        let dependency_command = Command::new(root_hash, dependency_command_hash)?;
        Self::traverse_helper(
            &root_dir,
            Some(config_path),
            excluded_project_dirs,
            |project_dir| {
                get_deps(&project_dir, &dependency_command).map(|x| {
                    x.into_iter()
                        .map(|x| String::from(path_to_str(&root_dir.join(x))))
                        .collect()
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

        project_dirs_alpha.sort();
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
                    return user_error_result(format!(
                        "Project directory {} does not exist",
                        path_to_str(p)
                    ));
                }
            }

            // TBD: Don't convert to string etc.
            ts.insert(String::from(path_to_str(project_dir)));

            for dep in &deps {
                // TBD: Don't convert to string etc.
                ts.add_dependency(dep, path_to_str(project_dir))
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
        root_dir: &PathBuf,
        config_path: Option<PathBuf>,
        excluded_project_dirs: &HashSet<PathBuf>,
        f: F,
    ) -> Result<Workspace>
    where
        F: Fn(&Path) -> Result<Vec<String>>,
    {
        let project_dirs_alpha = Self::get_project_dirs_alpha(&root_dir, &excluded_project_dirs)?;
        let project_dirs_topo = Self::topo_sort_project_dirs(&project_dirs_alpha, f)?;
        Ok(Workspace {
            root_dir: root_dir.to_path_buf(),
            config_path: config_path,
            project_dirs_alpha: project_dirs_alpha,
            project_dirs_topo: Some(project_dirs_topo),
        })
    }
}
