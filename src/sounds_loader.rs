use serde::Deserialize;
use ron::from_str;

use std::path::PathBuf;
use std::io::Read;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct List(Vec<Sound>);

#[derive(Debug, Deserialize, Default)]
pub struct Sound {
	name: String,
	wav: PathBuf,
	img: Option<PathBuf>,
}

impl List {
	pub fn new(file: PathBuf) -> Self {
		let mut s = String::new();
		let mut f = File::open(file).expect("Error opening file");
		f.read_to_string(&mut s).expect("Error reading file");
		from_str(&s).expect("Error parsing file")
	}
}

#[derive(Debug, Default)]
pub struct Sounds {
	sounds: Vec<Sound>,
	path: PathBuf,
}

impl Sounds {
	pub fn new(file: PathBuf) -> Self {
		let path = PathBuf::from(file.parent().unwrap());
		let List(sounds) = List::new(file);
		Self {
			sounds,
			path
		}
	}
}
