mod builder;
mod date;
mod template;
mod writer;

use anyhow::Result;
use std::fs;
use walkdir::WalkDir;

use std::fs::DirEntry;
use std::io;
use std::path::{Path, PathBuf};

use crate::builder::*;
use crate::builder::{Asset, CopyFile};
use crate::template::{parse, Post};
use crate::writer::*;

const CANT_PARSE: &str = "That's not unicode, can't parse path.";
pub fn to_local_path(path: &Path) -> io::Result<PathBuf> {
    Ok(PathBuf::from("./public").join(path.strip_prefix("assets").expect(CANT_PARSE)))
}

fn load_readme<P: AsRef<Path>>(path: P) -> Result<String> {
    Ok(template::custom_markdown_to_html(&fs::read_to_string(
        path,
    )?))
}

fn load_and_parse_post(dir_entry: std::io::Result<DirEntry>) -> Result<Post> {
    let entry = dir_entry?;
    if entry.file_type()?.is_dir() {
        return Err(io::Error::from(io::ErrorKind::InvalidInput).into());
    }
    parse(&fs::read_to_string(&entry.path())?).map_err(|e| e.into())
}

fn collect_others(dir_entry: walkdir::DirEntry) -> Result<Asset> {
    let entry = dir_entry;
    if entry.file_type().is_dir() {
        return Err(io::Error::from(io::ErrorKind::InvalidInput).into());
    }
    let path = entry.path().to_owned();
    Ok(Asset::Other(CopyFile { path }))
}

pub fn fetch_posts() -> Result<Vec<Post>> {
    fs::read_dir("./posts")?.map(load_and_parse_post).collect()
}

pub fn is_already_built(dir_entry: &walkdir::DirEntry) -> bool {
    let entry = dir_entry;
    if entry.file_type().is_dir() {
        return false;
    }
    match entry.path().extension() {
        Some(ext) => {
            ext == "png" && fs::metadata(to_local_path(entry.path()).expect(CANT_PARSE)).is_ok()
        }
        None => false,
    }
}

pub fn fetch_assets() -> Result<Vec<Asset>> {
    WalkDir::new("assets")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            let e_is_built = is_already_built(e);
            if e_is_built {
                println!("Excluding {:?}, already build.", e.path());
            }
            !e_is_built
        })
        .filter(|e| !e.file_type().is_dir())
        .map(collect_others)
        .collect()
}

pub struct Site {
    readme: String,
    posts: Vec<Post>,
    assets: Vec<Asset>,
}

impl Site {
    pub fn load_all() -> Result<Site> {
        Ok(Site {
            readme: load_readme(Path::new("./README.md"))?,
            posts: fetch_posts()?,
            assets: fetch_assets()?,
        })
    }
}

fn main() -> Result<()> {
    let site = Site::load_all()?;

    let mut built_site = BuiltSite::default();

    site.build(&mut built_site)?;

    built_site.write()?;

    Ok(())
}
