use directories::{BaseDirs, ProjectDirs};
use rdev::Key;
use ron;
use serde::*;
use std::collections::{HashMap};
use std::fs::{create_dir_all, read_to_string, File};
use std::hash::Hash;
use std::io::{Read, Write};
use std::path::PathBuf;

use log::info;

use crate::downloader;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct SoundRepo {
    pub zip_url: String,
    pub version_url: String,
}

impl SoundRepo {
    pub fn new<S: ToString, T: ToString>(zip_url: S, version_url: T) -> Self {
        Self {
            zip_url: zip_url.to_string(),
            version_url: version_url.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct DownloadedSoundRepo {
    pub version: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub loopback_device: Option<String>,
    #[cfg(feature = "autoloop")]
    pub autoloop: bool,
    pub hotkeys: HashMap<(String, String), Vec<Key>>,
    pub repos: Vec<(SoundRepo, Option<DownloadedSoundRepo>)>,
}

impl Default for Config {
    fn default() -> Self {
        println!("Defaulting config");
        Self {
            repos: vec![(SoundRepo::new("https://github.com/Mr-Llama-s-Wonderful-Soundboard/sounds/releases/download/latest/sounds.zip", "https://github.com/Mr-Llama-s-Wonderful-Soundboard/sounds/releases/download/latest/version.txt"), None)],
            input_device: None,
            output_device: None,
            loopback_device: None,
            #[cfg(feature = "autoloop")]
            autoloop: true,
            hotkeys: HashMap::new()
        }
    }
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
            println!("No config found");
            #[allow(unused_mut)]
            let mut s = Self::default();
            #[cfg(feature = "autoloop")]
            {
                s.autoloop = true;
            }
            println!("Done");
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
    pub repo: String,
    pub name: String,
    pub wav: PathBuf,
    pub img: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq, Hash, Clone)]
pub struct SoundRON {
    pub name: String,
    pub wav: PathBuf,
    pub img: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq, Hash, Clone)]
pub struct SoundsRON {
    pub default_img: PathBuf,
    pub name: String,
    pub sounds: Vec<SoundRON>,
}

#[derive(Debug, Default, Clone)]
pub struct SoundConfig {
    pub sounds: HashMap<String, HashMap<String, Sound>>,
    sounds_path: PathBuf,
}

impl SoundConfig {
    pub async fn load(config: &mut Config) -> Self {
        info!("Loading sounds");
        let basedirs = BaseDirs::new().expect("Error getting base dirs");
        let sounds_dir = basedirs.home_dir().join(".mlws/sounds");
        // let mut repos = HashMap::new();
        // let mut named_repos = HashMap::new();
        let mut sounds_hm = HashMap::new();
        for (repo, data) in &mut config.repos {
            let mut hm = HashMap::new();
            if !matches!(data, Some(x) if sounds_dir.join(&x.name).exists()) {
                downloader::download(repo, data, |p| {
                    match p {
                        downloader::Progress::Downloading(l, None) => {
                            println!("Downloading: {} bytes", l)
                        }
                        downloader::Progress::Downloading(l, Some(total)) => println!(
                            "Downloading[{}%]: {} bytes of {} bytes",
                            (l as f64 / total as f64 * 100.) as u8,
                            l,
                            total
                        ),
                        downloader::Progress::Installing() => println!("Installing"),
                        downloader::Progress::Done() => println!("Done"),
                    };
                })
                .await
            }
            let soundrepo_data = data.clone().unwrap();
            // named_repos.insert(soundrepo_data.name.clone(), repo.clone());
            let sounds: SoundsRON = ron::from_str(
                &read_to_string(sounds_dir.join(&soundrepo_data.name).join("sounds.ron"))
                    .expect("Error reading sounds.ron"),
            )
            .expect("Error parsing sounds");
            sounds.sounds.iter().for_each(|x| {
                hm.insert(
                    x.name.clone(),
                    Sound {
                        repo: soundrepo_data.name.clone(),
                        name: x.name.clone(),
                        wav: sounds_dir.join(&soundrepo_data.name).join(x.wav.clone()),
                        img: Some(
                            sounds_dir
                                .join(&soundrepo_data.name)
                                .join(x.img.clone().unwrap_or(sounds.default_img.clone())),
                        ),
                    },
                );
            });
            // hm.iter()
            //     .map(|(k, v)| {(format!("{}:{}", soundrepo_data.name, k), v)})
            //     .for_each(|(k, v)| {
            //         sounds_hm.insert(k.clone(), v.clone());
            //     });
            sounds_hm.insert(soundrepo_data.name, hm);
        }
        Self {
            sounds: sounds_hm,
            sounds_path: sounds_dir,
        }
        // if !sounds_dir.exists() || !sounds_config_path.exists() {
        //     let mut archive = Vec::new();
        //     println!("Downloading sounds from {}", SOUNDS_URL);
        //     reqwest::get(SOUNDS_URL).await
        //         .expect("Error getting sounds files")
        //         .read_to_end(&mut archive)
        //         .expect("Error reading archive");
        //     println!("Extracting sounds");
        //     zip::extract(Cursor::new(archive), &sounds_dir, true).expect("Error extracting archive")
        // }
        // let mut file = File::open(sounds_config_path).unwrap();
        // let mut buf = String::new();
        // file.read_to_string(&mut buf).expect("Error reading config");
        // let sounds_config: SoundRON = ron::from_str(&buf).expect("Error parsing sounds config");
        // let mut sounds = HashMap::new();
        // sounds_config.sounds.iter().for_each(|x| {
        //     sounds.insert(
        //         x.name.clone(),
        //         Sound {
        //             name: x.name.clone(),
        //             wav: sounds_dir.join(x.wav.clone()),
        //             img: Some(
        //                 sounds_dir.join(x.img.clone().unwrap_or(sounds_config.default_img.clone())),
        //             ),
        //         },
        //     );
        // });

        // info!("Done!");
        // Self {
        //     sounds,
        //     sounds_path: sounds_dir,
        // }
    }

    pub fn get(&self, repo: &String, name: &String) -> Option<&Sound> {
        self.sounds.get(repo).and_then(|x| x.get(name))
    }

    pub fn json_sounds(&self) -> HashMap<String, Vec<String>> {
        self.sounds
            .iter()
            .map(|(k, v)| (k.clone(), v.keys().cloned().collect()))
            .collect()
    }
}
