use std::env;
use std::path::{Path, PathBuf};

pub const RES_FOLDER: &str = "res";

pub fn get_resource<P: AsRef<Path>>(path: P) -> Box<PathBuf>
{
	Box::new(Path::new(env!("OUT_DIR")).join(RES_FOLDER).join(path))
}

pub fn get_bytes<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>>
{
	std::fs::read(get_resource(path).to_str().unwrap())
}