use crate::config::{Config, ConfigHash};
use crate::deps::get_deps;
use crate::scripting::command::Command;

use std::collections::HashSet;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Component, Path, PathBuf};
use topological_sort::TopologicalSort;

const WORKSPACE_CONFIG_FILE_NAME: &str = "rws-workspace.yaml";

pub struct Workspace {
    pub config_path: PathBuf,
    pub root_dir: PathBuf,
    pub project_dirs_alpha: Vec<PathBuf>,
    pub project_dirs_topo: Vec<PathBuf>,
}

impl Workspace {
    pub fn find(search_dir: &Path) -> std::io::Result<Workspace> {
        let x = search_dir.join(WORKSPACE_CONFIG_FILE_NAME);
        if x.exists() {
            let root_dir = x.parent().unwrap().to_path_buf();
            return Workspace::traverse(x, root_dir);
        }

        match search_dir.parent() {
            Some(p) => Self::find(p),
            None => Err(Error::new(ErrorKind::Other, "Could not find workspace")),
        }
    }

    fn traverse(config_path: PathBuf, root_dir: PathBuf) -> std::io::Result<Workspace> {
        let config = Config::read_yaml_file(&config_path)?;
        let root_hash = config.as_hash().unwrap();
        let excluded_project_dirs = root_hash
            .as_str_vec("excluded-projects")
            .unwrap()
            .into_iter()
            .map(|x| root_dir.join(x))
            .collect::<HashSet<_>>();
        let dependencies_hash_opt = root_hash.as_hash("dependencies");
        let dependency_command_hash_opt = root_hash.as_hash("dependency-command");
        match (&dependencies_hash_opt, &dependency_command_hash_opt) {
            (Some(_), Some(_)) => {
                panic!("Invalid: cannot specify both dependencies and dependency-command")
            }
            (Some(dependencies_hash), None) => Workspace::traverse_with_dependencies(
                config_path,
                root_dir,
                &excluded_project_dirs,
                &dependencies_hash,
            ),
            (None, Some(dependency_command_hash)) => Workspace::traverse_with_dependency_command(
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
    ) -> std::io::Result<Workspace> {
        Workspace::traverse_helper(
            config_path,
            &root_dir,
            excluded_project_dirs,
            |project_dir| {
                let project_name = match project_dir.components().last().unwrap() {
                    Component::Normal(s) => s,
                    _ => panic!("Unimplemented"),
                };
                match dependency_command_hash.as_vec(project_name.to_str().unwrap()) {
                    Some(v) => (0..v.len())
                        .into_iter()
                        .map(|i| {
                            String::from(root_dir.join(v.as_str(i).unwrap()).to_str().unwrap())
                        })
                        .collect(),
                    None => Vec::new(),
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
    ) -> std::io::Result<Workspace> {
        let dependency_command = Command::new(root_hash, dependency_command_hash);
        Workspace::traverse_helper(
            config_path,
            &root_dir,
            excluded_project_dirs,
            |project_dir| {
                get_deps(&project_dir, &dependency_command)
                    .unwrap()
                    .into_iter()
                    .map(|x| String::from(root_dir.join(x).to_str().unwrap()))
                    .collect::<Vec<_>>()
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

    fn topo_sort_project_dirs<F>(project_dirs_alpha: &Vec<PathBuf>, f: F) -> Vec<PathBuf>
    where
        F: Fn(&Path) -> Vec<String>,
    {
        let mut ts = TopologicalSort::<String>::new();
        for project_dir in project_dirs_alpha {
            let deps = f(project_dir);

            // TBD: Don't convert to string etc.
            ts.insert(String::from(project_dir.to_str().unwrap()));

            for dep in deps {
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

        project_dirs_topo
    }

    fn traverse_helper<F>(
        config_path: PathBuf,
        root_dir: &PathBuf,
        excluded_project_dirs: &HashSet<PathBuf>,
        f: F,
    ) -> std::io::Result<Workspace>
    where
        F: Fn(&Path) -> Vec<String>,
    {
        let project_dirs_alpha =
            Workspace::get_project_dirs_alpha(&root_dir, &excluded_project_dirs).unwrap();
        let project_dirs_topo = Workspace::topo_sort_project_dirs(&project_dirs_alpha, f);

        Ok(Workspace {
            config_path: config_path,
            root_dir: root_dir.to_path_buf(),
            project_dirs_alpha: project_dirs_alpha,
            project_dirs_topo: project_dirs_topo,
        })
    }
}
