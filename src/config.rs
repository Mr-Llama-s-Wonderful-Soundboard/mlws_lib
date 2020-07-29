use directories::{BaseDirs, ProjectDirs};
use reqwest;
use ron;
use serde::*;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::fs::{create_dir_all, File};
use std::hash::Hash;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use zip_extract as zip;
use crossbeam_channel;


use log::info;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub loopback_device: Option<String>,
    #[cfg(feature = "autoloop")]
    pub autoloop: bool,
    pub hotkeys: HashMap<String, (u32, u32)>,
}

impl Config {
    pub fn load() -> Self {
        let project_dirs = ProjectDirs::from("", "", "MrLlamasWonderfulSoundboard").unwrap();
        let config_dir = project_dirs.config_dir();
        if !config_dir.exists() {
            create_dir_all(config_dir).expect("Error creating config dir");
        }
        let config_path = config_dir.join("config.ron");
        if config_path.exists() {
            let mut file = File::open(config_path).unwrap();
            let mut buf = String::new();
            file.read_to_string(&mut buf).expect("Error reading config");
            ron::from_str(&buf).expect("Error parsing config")
        } else {
            #[allow(unused_mut)]
            let mut s = Self::default();
            #[cfg(feature = "autoloop")]
            {
                s.autoloop = true;
            }
            s.save();
            s
        }
    }

    pub fn save(&self) {
        let project_dirs = ProjectDirs::from("", "", "MrLlamasWonderfulSoundboard").unwrap();
        let config_dir = project_dirs.config_dir();

        let config_path = config_dir.join("config.ron");
        let mut file = File::create(config_path).unwrap();
        write!(
            file,
            "{}",
            ron::to_string(self).expect("Error writing config")
        )
        .expect("Error writing config");
        file.flush().expect("Error flushing config");
    }
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq, Hash, Clone)]
pub struct Sound {
    pub name: String,
    pub wav: PathBuf,
    pub img: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq, Hash, Clone)]
pub struct SoundRON {
    pub default_img: PathBuf,
    pub sounds: Vec<Sound>,
}

#[derive(Debug, Default, Clone)]
pub struct SoundConfig {
    pub sounds: HashMap<String, Sound>,
}

const SOUNDS_URL: &str =
    "https://github.com/Mr-Llama-s-Wonderful-Soundboard/sounds/archive/master.zip";

impl SoundConfig {
    pub fn load() -> Self {
        info!("Loading sounds");
        let basedirs = BaseDirs::new().expect("Error getting base dirs");
        let sounds_dir = basedirs.home_dir().join(".mlws");
        let sounds_config_path = sounds_dir.join("sounds.ron");
        if !sounds_dir.exists() || !sounds_config_path.exists() {
            let mut archive = Vec::new();
            info!("Downloading sounds from {}", SOUNDS_URL);
            reqwest::blocking::get(SOUNDS_URL)
                .expect("Error getting sounds files")
                .read_to_end(&mut archive)
                .expect("Error reading archive");
            info!("Extracting sounds");
            zip::extract(Cursor::new(archive), &sounds_dir, true).expect("Error extracting archive")
        }
        let mut file = File::open(sounds_config_path).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).expect("Error reading config");
        let sounds_config: SoundRON = ron::from_str(&buf).expect("Error parsing sounds config");
        let mut sounds = HashMap::new();
        sounds_config.sounds.iter().for_each(|x| {
            sounds.insert(
                x.name.clone(),
                Sound {
                    name: x.name.clone(),
                    wav: sounds_dir.join(x.wav.clone()),
                    img: Some(
                        sounds_dir.join(x.img.clone().unwrap_or(sounds_config.default_img.clone())),
                    ),
                },
            );
        });

        info!("Done!");
        Self { sounds }
    }

    

    pub fn download_and_extract() {
        let basedirs = BaseDirs::new().expect("Error getting base dirs");
        let sounds_dir = basedirs.home_dir().join(".mlws");
        let mut archive = Vec::new();
        info!("Downloading sounds from {}", SOUNDS_URL);
        reqwest::blocking::get(SOUNDS_URL)
            .expect("Error getting sounds files")
            .read_to_end(&mut archive)
            .expect("Error reading archive");
        info!("Extracting sounds");
        zip::extract(Cursor::new(archive), &sounds_dir, true).expect("Error extracting archive");
    }

    pub fn get(&self, name: &String) -> Option<&Sound> {
        self.sounds.get(name)
    }
}

