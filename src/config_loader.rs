use std::path::PathBuf;
use serde::Deserialize;
use std::{collections::HashMap};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub paths: Paths,
    #[allow(unused)]
    pub kernel: Kernel,
    #[allow(unused)]
    pub programs: HashMap<String, PathBuf>,
    #[allow(unused)]
    pub files: HashMap<String, PathBuf>, // keys like "text-file", "bmp-image"
}

#[derive(Debug, Deserialize)]
pub struct Paths {
    #[allow(unused)]
    pub libs: PathBuf,
    pub build: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Kernel {
    #[allow(unused)]
    pub path: PathBuf,
}


pub fn find_project_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("nex.toml").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn load_config() -> Option<Config> {
    let root = find_project_root()?;
    let path = root.join("nex.toml");
    let text = std::fs::read_to_string(path).unwrap();
    Some(toml::from_str(&text).unwrap())
}
