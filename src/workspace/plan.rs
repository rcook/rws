use crate::error::{user_error, user_error_result, Result};
use crate::os::with_working_dir;
use crate::os::{get_base_name, path_to_str};

use super::internal::{DependencySource, Workspace};

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use topological_sort::TopologicalSort;

/// A build plan for a workspace
pub struct Plan {
    /// Workspace
    pub workspace: Workspace,
    /// Project directories in alphabetical order
    pub project_dirs_alpha: Vec<PathBuf>,
    /// Project directories in topological order
    pub project_dirs_topo: Option<Vec<PathBuf>>,
}

impl Plan {
    /// Create a plan from a workspace
    pub fn resolve(workspace: Workspace) -> Result<Self> {
        let project_dirs_alpha = Self::get_project_dirs_alpha(
            &workspace.workspace_dir,
            &workspace.excluded_project_dirs,
        )?;

        let project_dirs_topo = match &workspace.dependency_source {
            DependencySource::Hash(hash) => Some(Self::topo_sort_project_dirs(
                &project_dirs_alpha,
                |project_dir| {
                    let project_name = get_base_name(project_dir)
                        .ok_or_else(|| user_error("Invalid project directory"))?;
                    match hash.get(project_name).and_then(|x| x.into_vec()) {
                        Some(v) => (0..v.len())
                            .into_iter()
                            .map(|i| {
                                v.get(i)
                                    .into_string()
                                    .ok_or_else(|| {
                                        user_error(format!(
                                            "Invalid dependency for project {}",
                                            project_name
                                        ))
                                    })
                                    .map(|s| {
                                        String::from(path_to_str(&workspace.workspace_dir.join(s)))
                                    })
                            })
                            .collect(),
                        None => Ok(Vec::new()),
                    }
                },
            )?),
            DependencySource::ScriptCommand(command) => Some(Self::topo_sort_project_dirs(
                &project_dirs_alpha,
                |project_dir| {
                    with_working_dir(project_dir, || command.eval())?.map(|x: Vec<String>| {
                        x.into_iter()
                            .map(|x| String::from(path_to_str(&workspace.workspace_dir.join(x))))
                            .collect()
                    })
                },
            )?),
            DependencySource::None => None,
        };
        Ok(Self {
            workspace: workspace,
            project_dirs_alpha: project_dirs_alpha,
            project_dirs_topo: project_dirs_topo,
        })
    }

    fn get_project_dirs_alpha(
        workspace_dir: &Path,
        excluded_project_dirs: &HashSet<PathBuf>,
    ) -> std::io::Result<Vec<PathBuf>> {
        let mut project_dirs_alpha = Vec::new();
        for entry in fs::read_dir(&workspace_dir)? {
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
            let project_dirs_alpha_set = project_dirs_alpha.iter().collect::<HashSet<_>>();
            for dep in &deps {
                let p = Path::new(dep);
                if !p.is_dir() {
                    return user_error_result(format!(
                        "Project directory {} does not exist",
                        path_to_str(p)
                    ));
                }

                // TBD: Lots of copying happening here!
                if !project_dirs_alpha_set.contains(&p.to_path_buf()) {
                    return user_error_result(format!(
                        "Project dependency {} is not a valid Git repository",
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
}
