use crate::config::Config;
use crate::deps::get_deps;
use crate::scripting::command::Command;

use std::collections::HashSet;
use std::fs;
use std::io::{ Error, ErrorKind };
use std::path::{ Path, PathBuf };
use topological_sort::TopologicalSort;

const WORKSPACE_CONFIG_FILE_NAME: &str = "rws-workspace.yaml";

pub struct Workspace {
    pub config_path: PathBuf,
    pub root_dir: PathBuf,
    pub project_dirs_alpha: Vec<PathBuf>,
    pub project_dirs_topo: Vec<PathBuf>
}

impl Workspace {
    pub fn find(search_dir: &Path) -> std::io::Result<Workspace> {
        let x = search_dir.join(WORKSPACE_CONFIG_FILE_NAME);
        if x.exists() {
            let root_dir = x.parent().unwrap().to_path_buf();
            return Workspace::traverse(x, root_dir)
        }

        match search_dir.parent() {
            Some(p) => Self::find(p),
            None => Err(Error::new(ErrorKind::Other, "Could not find workspace"))
        }
    }

    fn traverse(config_path: PathBuf, root_dir: PathBuf) -> std::io::Result<Workspace> {
        let config = Config::read_yaml_file(&config_path)?;

        let root_hash = config.as_hash().unwrap();

        let excluded_project_dirs = root_hash
            .as_str_vec("excluded-projects").unwrap()
            .into_iter()
            .map(|x| root_dir.join(x))
            .collect::<HashSet<_>>();

        let dependency_command = root_hash.as_hash("dependency-command").as_ref().map(|x| Command::new(&root_hash, x));

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

        let mut ts = TopologicalSort::<String>::new();
        for project_dir in &project_dirs_alpha {
            let deps = match &dependency_command {
                Some(c) => get_deps(&project_dir, c).unwrap()
                                .into_iter()
                                .map(|x| root_dir.join(x))
                                .collect::<Vec<_>>(),
                None => Vec::new()
            };

            // TBD: Don't convert to string etc.
            ts.insert(String::from(project_dir.to_str().unwrap()));

            for dep in deps {
                // TBD: Don't convert to string etc.
                ts.add_dependency(String::from(dep.to_str().unwrap()), project_dir.to_str().unwrap())
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

        Ok(Workspace {
            config_path: config_path,
            root_dir: root_dir,
            project_dirs_alpha: project_dirs_alpha,
            project_dirs_topo: project_dirs_topo
        })
    }
}
