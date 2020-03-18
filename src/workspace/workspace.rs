use crate::config::{ConfigHash, ConfigObject};
use crate::error::{user_error, user_error_result, Result};
use crate::scripting::command::Command;

use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

const WORKSPACE_CONFIG_FILE_NAME: &str = "rws-workspace.yaml";

/// Source of dependency information
pub enum DependencySource {
    /// Dependencies specified in configuration file
    Hash(ConfigHash),
    /// Dependencies generated by external script command
    Command(Command),
    /// No dependency information provided
    None,
}

/// Workspace information derived from file system and configuration file
pub struct Workspace {
    /// Workspace directory
    pub workspace_dir: PathBuf,
    /// Configuration path
    pub config_path: Option<PathBuf>,
    /// Excluded project directories
    pub excluded_project_dirs: HashSet<PathBuf>,
    /// Source of dependency information
    pub dependency_source: DependencySource,
    /// Workspace initialization command
    pub init_command: Option<Command>,
}

impl Workspace {
    /// Constructor
    pub fn new(workspace_dir: Option<PathBuf>, config_path: Option<PathBuf>) -> Result<Self> {
        match (workspace_dir, config_path) {
            (Some(d), Some(c)) => Self::known(d, Some(c)),
            (Some(d), None) => {
                let p = d.join(WORKSPACE_CONFIG_FILE_NAME);
                Self::known(d, if p.exists() { Some(p) } else { None })
            }
            (None, Some(c)) => Self::known(
                c.parent()
                    .ok_or_else(|| user_error("Invalid config path"))?
                    .to_path_buf(),
                Some(c),
            ),
            (None, None) => Self::find(&env::current_dir()?),
        }
    }

    fn known(workspace_dir: PathBuf, config_path: Option<PathBuf>) -> Result<Self> {
        match &config_path {
            Some(c) => match ConfigObject::read_config_file(&c)? {
                Some(config) => Self::read_config(workspace_dir, c.to_path_buf(), config),
                None => Ok(Self {
                    workspace_dir: workspace_dir,
                    config_path: config_path,
                    excluded_project_dirs: HashSet::new(),
                    dependency_source: DependencySource::None,
                    init_command: None,
                }),
            },
            None => Ok(Self {
                workspace_dir: workspace_dir,
                config_path: config_path,
                excluded_project_dirs: HashSet::new(),
                dependency_source: DependencySource::None,
                init_command: None,
            }),
        }
    }

    fn find(search_dir: &Path) -> Result<Self> {
        let mut p = search_dir;
        loop {
            let config_path = p.join(WORKSPACE_CONFIG_FILE_NAME);
            if config_path.exists() {
                match ConfigObject::read_config_file(&config_path)? {
                    Some(config) => return Self::read_config(p.to_path_buf(), config_path, config),
                    None => {
                        return Ok(Self {
                            workspace_dir: p.to_path_buf(),
                            config_path: Some(config_path),
                            excluded_project_dirs: HashSet::new(),
                            dependency_source: DependencySource::None,
                            init_command: None,
                        })
                    }
                }
            }
            match p.parent() {
                Some(parent) => p = parent,
                None => {
                    return Ok(Self {
                        workspace_dir: search_dir.to_path_buf(),
                        config_path: None,
                        excluded_project_dirs: HashSet::new(),
                        dependency_source: DependencySource::None,
                        init_command: None,
                    })
                }
            }
        }
    }

    fn read_config(
        workspace_dir: PathBuf,
        config_path: PathBuf,
        config_object: ConfigObject,
    ) -> Result<Self> {
        let root_hash = config_object
            .into_hash()
            .ok_or_else(|| user_error("Invalid config hash"))?;
        let excluded_project_dirs = root_hash
            .get("excluded-projects")
            .and_then(|x| x.into_vec())
            .map(|x| {
                let mut values = Vec::new();
                for i in 0..x.len() {
                    values.push(x.get(i).into_string().expect("into_string failed"))
                }
                values
            })
            .unwrap_or_else(|| Vec::new())
            .into_iter()
            .map(|x| workspace_dir.join(x))
            .collect::<HashSet<_>>();
        let dependencies_hash_opt = root_hash.get("dependencies").and_then(|x| x.into_hash());
        let dependency_command_hash_opt = root_hash
            .get("dependency-command")
            .and_then(|x| x.into_hash());
        let init_command = match root_hash.get("init-command").and_then(|x| x.into_hash()) {
            Some(h) => Some(Command::new(&root_hash, &h)?),
            None => None,
        };

        match (dependencies_hash_opt, &dependency_command_hash_opt) {
            (Some(_), Some(_)) => user_error_result("Must specify at most one of \"dependencies\" and \"dependency-command\" in workspace configuration"),
            (Some(dependencies_hash), None) => Ok(Self {
                workspace_dir: workspace_dir,
                config_path: Some(config_path),
                excluded_project_dirs: excluded_project_dirs,
                dependency_source: DependencySource::Hash(dependencies_hash),
                init_command:init_command
            }),
            (None, Some(dependency_command_hash)) => Ok(Self {
                workspace_dir: workspace_dir,
                config_path: Some(config_path),
                excluded_project_dirs: excluded_project_dirs,
                dependency_source: DependencySource::Command(Command::new(&root_hash, &dependency_command_hash)?),
                init_command:init_command
            }),
            (None, None) => Ok(Self {
                workspace_dir: workspace_dir,
                config_path: Some(config_path),
                excluded_project_dirs: excluded_project_dirs,
                dependency_source: DependencySource::None,
                init_command:init_command
            })
        }
    }
}
