use std::{path::PathBuf, process::Command};

use convert_case::{Case, Casing};
use faker_rand::en_us::names::FirstName;

#[derive(Debug)]
pub struct Config {
    pub rails_path: String,
    pub base_dir: String,
    pub app_name: String,
    pub num_packages: usize,
    pub codeowners_dotslash_path: String,
}

impl Config {
    pub fn app_dir(&self) -> PathBuf {
        PathBuf::from(&self.base_dir).join(&self.app_name)
    }
}

pub fn packages(num: &usize) -> Vec<String> {
    let mut packages = vec![];
    for _ in 0..*num {
        let name = rand::random::<FirstName>()
        .to_string()
        .to_case(Case::Snake);
        packages.push(name);
    }
    packages
}

pub fn build_app(config: Config) -> anyhow::Result<()> {
    Command::new(&config.rails_path)
        .arg("new")
        .arg(&config.app_dir())
        .output()?;
    std::fs::write(
        config.app_dir().join("config/code_ownership.yml"),
        DEFAULT_CODE_OWNERSHIP_YML,
    )?;
    for pack in packages(&config.num_packages) {
        let pack_config = PackConfig {
            config: &config,
            name: &pack,
            ownership: PackOwnership::Directory,
        };      
        build_pack(&pack_config)?;
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum PackOwnership {
    Directory,
    FileAnnotation,
    TeamConfig,
    PackConfig,
}
struct PackConfig<'a> {
    config: &'a Config,
    name: &'a str,
    ownership: PackOwnership,
}
impl<'a> PackConfig<'a> {
    fn team_name(&self) -> String {
        format!("{}-team", self.name)
    }
    fn pack_path(&self) -> PathBuf {
        self.config.app_dir().join("packs").join(self.name)
    }

    fn relative_pack_path(&self) -> PathBuf {
        self.pack_path().strip_prefix(&self.config.app_dir()).unwrap().to_path_buf()
    }
}

fn build_pack(pack_config: &PackConfig) -> anyhow::Result<()> {
    let team_name = pack_config.team_name();
    std::fs::create_dir_all(pack_config.config.app_dir().join("config/teams").join(&team_name))?;
    
    let mut team_config = String::new();
    team_config.push_str(&format!("name: {}\n", &team_name));
    if pack_config.ownership == PackOwnership::TeamConfig {
        team_config.push_str(&format!("\nowned_globs:\n  - \"{}/**\"\n", pack_config.relative_pack_path().display()));
    } 
    std::fs::write(
        pack_config.config.app_dir().join("config/teams").join(format!("{}-team.yml", pack_config.name)),
        team_config,
    )?;
    std::fs::create_dir_all(pack_config.pack_path())?;

    if pack_config.ownership == PackOwnership::PackConfig {
        std::fs::write(pack_config.pack_path().join("package.yml"), 
        format!("owner: {}\n", pack_config.team_name()))?;
    }

    if pack_config.ownership == PackOwnership::Directory {
        std::fs::write(pack_config.pack_path().join(".codeowner"), 
        format!("{}\n", pack_config.team_name()))?;
    }
    Ok(())
}

const DEFAULT_CODE_OWNERSHIP_YML: &str = "
---
owned_globs:
  - \"{app,components,config,frontend,lib,packs,spec}/**/*.{rb,rake,js,jsx,ts,tsx,json,yml}\"
unowned_globs:
  - config/code_ownership.yml
javascript_package_paths:
  - javascript/packages/**
vendored_gems_path: gems
team_file_glob:
  - config/teams/**/*.yml
        ";
