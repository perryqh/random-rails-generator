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
    pub pks_dotslash_path: String,
}

impl Config {
    pub fn app_dir(&self) -> PathBuf {
        PathBuf::from(&self.base_dir).join(&self.app_name)
    }
}

fn random_name() -> String {
    rand::random::<FirstName>()
        .to_string()
        .to_case(Case::Snake)
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
}

fn packages(num: &usize) -> Vec<String> {
    (0..*num).map(|_| random_name()).collect()
}

pub fn build_app(config: Config) -> anyhow::Result<()> {
    setup_rails_app(&config)?;
    setup_dotslash_tools(&config)?;
    setup_infra_team(&config)?;

    packages(&config.num_packages)
        .into_iter()
        .map(|pack| {
            let ownership = PackOwnership::random();
            let pack_config = PackConfig::new(&config, &pack, ownership);
            build_pack(&pack_config)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

#[derive(Debug, PartialEq)]
enum PackOwnership {
    Directory,
    FileAnnotation,
    TeamConfig,
    PackConfig,
}

impl PackOwnership {
    fn random() -> Self {
        match rand::random::<u8>() % 4 {
            0 => Self::Directory,
            1 => Self::FileAnnotation,
            2 => Self::TeamConfig,
            _ => Self::PackConfig,
        }
    }
}

struct PackConfig<'a> {
    config: &'a Config,
    name: &'a str,
    ownership: PackOwnership,
}

impl<'a> PackConfig<'a> {
    fn new(config: &'a Config, name: &'a str, ownership: PackOwnership) -> Self {
        Self {
            config,
            name,
            ownership,
        }
    }
    fn team_name(&self) -> String {
        format!("{}-team", self.name)
    }
    fn pack_path(&self) -> PathBuf {
        self.config.app_dir().join("packs").join(self.name)
    }

    fn relative_pack_path(&self) -> PathBuf {
        self.pack_path()
            .strip_prefix(&self.config.app_dir())
            .unwrap()
            .to_path_buf()
    }
}

fn build_pack(pack_config: &PackConfig) -> anyhow::Result<()> {
    let team_name = pack_config.team_name();
    match setup_team_directory(pack_config, &team_name)? {
        TeamSetupResult::Success => {}
        TeamSetupResult::AlreadyExists => {
            return Ok(());
        }
    }
    write_team_config(pack_config, &team_name)?;
    setup_pack_directory(pack_config)?;
    write_ownership_files(pack_config)?;
    generate_code_files(pack_config)?;
    Ok(())
}

fn write_code_file(
    dir_path: &PathBuf,
    name: &str,
    team: &str,
    annotate: bool,
) -> anyhow::Result<()> {
    let file_path = dir_path.join(format!("{}.rb", name));
    let mut file_contents = String::new();
    if annotate {
        file_contents.push_str(&format!("# @team {}\n", team));
    }
    file_contents.push_str(&format!("class {}\n{}\nend\n", name, FILE_CONTENTS));

    Ok(std::fs::write(file_path, file_contents)?)
}

const CODE_DIRECTORIES: &[&str] = &[
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z",
];

const FILE_CONTENTS: &str = "
  def method_1
    puts 'hello'
  end  

  def method_2
      puts 'hello 2'
  end

  def method_3
     puts 'met'
  end

  The borrow checker is an essential feature of the Rust language and part of what makes Rust Rust. It helps you (or forces you) to manage ownership. As chapter four of “The Rust Programming Language” puts it, “Ownership is Rust’s most unique feature, and it enables Rust to make memory safety guarantees without needing a garbage collector.”

In the last few sentences, we’ve mentioned ownership, borrow checker, and garbage collectors. There’s a lot to unpack there, so let’s break it down a bit. In this guide, we’ll look at what the borrow checker does for us (and what it stops us from doing), what guarantees it gives us, and how it compares to other forms of memory management.

I’ll assume that you have some experience writing code in higher-level languages such as Python, JavaScript, and C#, but not necessarily that you’re familiar with how computer memory works.

Your programs have access to two kinds of memory where they can store values: the stack and the heap. These differ in several ways, but for our sake, the most important difference is that data stored on the stack must have a known, fixed size. Data on the heap can be of any arbitrary size.

What do I mean by size? Size refers to how many bytes it takes to store the data. In broad terms, certain data types, such as Booleans, characters, and integers, have a fixed size. These are easy to put on the stack. On the other hand, data types such as strings, lists, and other collections can be of any arbitrary size. As such, they cannot be stored on the stack. We must instead use the heap.

Because data of arbitrary size can be stored on the heap, the computer needs to find a chunk of memory large enough to fit whatever we’re looking to store. This is time-consuming, and the program doesn’t have direct access to the data as with the stack. Instead, it’s left with a pointer to where the data is stored.

A pointer is pretty much what it says on the tin: it points to some memory address on the heap where the data you’re looking for can be found. There are several pointer tutorials available on the web, and which one works for you depends on your background.

What’s the point of having these two different memory stores? Because of the way the stack works, data access on the stack is fast and easy but requires the data to conform to certain standards. The heap is slower but more versatile and is thus useful when you can’t use the stack.
";

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
const DEFAULT_DEVOPS_TEAM_YML: &str = "
name: devops
github:
  team: '@devops'
  members:
  - devops member
owned_globs:
- app/**
- config/application.rb
- config/boot.rb
- config/cable.yml
- config/database.yml
- config/environment.rb
- config/environments/development.rb
- config/environments/production.rb
- config/environments/test.rb
- config/importmap.rb
- config/initializers/assets.rb
- config/initializers/content_security_policy.rb
- config/initializers/filter_parameter_logging.rb
- config/initializers/inflections.rb
- config/initializers/permissions_policy.rb
- config/locales/en.yml
- config/puma.rb
- config/routes.rb
- config/storage.yml
- config/cache.yml
- config/deploy.yml
- config/queue.yml
- config/recurring.yml- config/cache.yml
- config/deploy.yml
- config/queue.yml
- config/recurring.yml

";

fn setup_rails_app(config: &Config) -> anyhow::Result<()> {
    Command::new(&config.rails_path)
        .arg("new")
        .arg(&config.app_dir())
        .output()?;

    std::fs::write(
        config.app_dir().join("config/code_ownership.yml"),
        DEFAULT_CODE_OWNERSHIP_YML,
    )?;

    Ok(())
}

fn setup_dotslash_tools(config: &Config) -> anyhow::Result<()> {
    let dotslash_dir = config.app_dir().join(".dotslash");
    std::fs::create_dir_all(&dotslash_dir)?;

    // Setup PKS tool
    let pks_path = dotslash_dir.join("pks");
    let pks_bytes = reqwest::blocking::get(&config.pks_dotslash_path)?.bytes()?;
    std::fs::write(&pks_path, pks_bytes)?;
    make_executable(&pks_path)?;

    // Setup codeowners tool
    let codeowners_path = dotslash_dir.join("codeowners-rs");
    let codeowners_bytes = reqwest::blocking::get(&config.codeowners_dotslash_path)?.bytes()?;
    std::fs::write(&codeowners_path, codeowners_bytes)?;
    make_executable(&codeowners_path)?;

    Ok(())
}

fn make_executable(path: &PathBuf) -> anyhow::Result<()> {
    Command::new("chmod").arg("+x").arg(path).output()?;
    Ok(())
}

fn setup_infra_team(config: &Config) -> anyhow::Result<()> {
    let team_name = "infra";
    let team_dir = config.app_dir().join("config/teams").join(team_name);
    std::fs::create_dir_all(&team_dir)?;

    std::fs::write(
        team_dir.join(format!("{}.yml", team_name)),
        DEFAULT_DEVOPS_TEAM_YML,
    )?;

    Ok(())
}

enum TeamSetupResult {
    Success,
    AlreadyExists,
}

fn setup_team_directory(
    pack_config: &PackConfig,
    team_name: &str,
) -> anyhow::Result<TeamSetupResult> {
    let team_dir = pack_config
        .config
        .app_dir()
        .join("config/teams")
        .join(team_name);
    if team_dir.exists() {
        return Ok(TeamSetupResult::AlreadyExists);
    }
    std::fs::create_dir_all(&team_dir)?;
    Ok(TeamSetupResult::Success)
}

fn write_team_config(pack_config: &PackConfig, team_name: &str) -> anyhow::Result<()> {
    let team_config = generate_team_config(pack_config, team_name);
    let config_path = pack_config
        .config
        .app_dir()
        .join("config/teams")
        .join(team_name)
        .join(format!("{}-team.yml", team_name));

    std::fs::write(config_path, team_config)?;
    Ok(())
}

fn generate_team_config(pack_config: &PackConfig, team_name: &str) -> String {
    let mut config = format!(
        "name: {}\ngithub:\n  team: '@{}'\n  members:\n    - {} member\n",
        team_name, team_name, team_name
    );

    if pack_config.ownership == PackOwnership::TeamConfig {
        config.push_str(&format!(
            "\nowned_globs:\n  - \"{}/**\"\n",
            pack_config.relative_pack_path().display()
        ));
    }

    config
}

fn setup_pack_directory(pack_config: &PackConfig) -> anyhow::Result<()> {
    std::fs::create_dir_all(pack_config.pack_path())?;
    Ok(())
}

fn write_ownership_files(pack_config: &PackConfig) -> anyhow::Result<()> {
    match pack_config.ownership {
        PackOwnership::PackConfig => {
            std::fs::write(
                pack_config.pack_path().join("package.yml"),
                format!("owner: {}\n", pack_config.team_name()),
            )?;
        }
        PackOwnership::Directory => {
            std::fs::write(
                pack_config.pack_path().join(".codeowner"),
                format!("{}\n", pack_config.team_name()),
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn generate_code_files(pack_config: &PackConfig) -> anyhow::Result<()> {
    let annotate = pack_config.ownership == PackOwnership::FileAnnotation;
    let team_name = pack_config.team_name();

    for dir in CODE_DIRECTORIES {
        let dir_path = pack_config.pack_path().join("app/services").join(dir);
        std::fs::create_dir_all(&dir_path)?;

        for _ in 0..30 {
            write_code_file(&dir_path, &random_name(), &team_name, annotate)?;
        }
    }

    Ok(())
}
