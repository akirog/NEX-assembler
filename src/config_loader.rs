use std::path::PathBuf;
use serde::Deserialize;
use std::{collections::HashMap};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub paths: Paths,
    pub kernel: Kernel,
    pub programs: HashMap<String, PathBuf>,
    pub files: HashMap<String, PathBuf>, // keys like "text-file", "bmp-image"
}

#[derive(Debug, Deserialize)]
pub struct Paths {
    pub libs: PathBuf,
    pub build: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Kernel {
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

pub fn load_config() -> Config {
    let root = find_project_root().unwrap();
    let path = root.join("nex.toml");
    let text = std::fs::read_to_string(path).unwrap();
    toml::from_str(&text).unwrap()
}
