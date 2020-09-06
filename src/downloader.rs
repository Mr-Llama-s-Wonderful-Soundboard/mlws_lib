use crate::config::{DownloadedSoundRepo, SoundsRON, SoundRepo};

use std::fs::{create_dir_all, read_to_string, rename};
use std::io::Cursor;

use directories::BaseDirs;
use reqwest;
use zip_extract as zip;

use ron::from_str;

pub enum Status {
    Latest(String),
    Updatable(Option<String>, String),
}

pub enum Progress {
    Downloading(u64, Option<u64>),
    Installing(),
    Done(),
}

pub async fn status(repo: &SoundRepo, data: &Option<DownloadedSoundRepo>) -> Status {
    let current_version = data.clone().map(|x| x.version);
    let latest_version = reqwest::get(&repo.version_url)
        .await
        .expect("Error getting latest version")
        .text()
        .await
        .expect("Version should be string");
    if Some(latest_version.clone()) == current_version {
        Status::Latest(latest_version)
    } else {
        Status::Updatable(current_version, latest_version)
    }
}

pub async fn download<F: Fn(Progress)>(
    repo: &SoundRepo,
    data: &mut Option<DownloadedSoundRepo>,
    on_progress: F,
) {
    let status = status(repo, data).await;
    match status {
        Status::Latest(_) => (), // Nothing to do,
        Status::Updatable(_, latest) => {
            let mut response = reqwest::get(&repo.zip_url)
                .await
                .expect("Error getting repo zip");
            let content_length = response.content_length();
            on_progress(Progress::Downloading(0, content_length));
            let mut content = Vec::new();
            let mut download_len = 0;
            while let Ok(Some(c)) = response.chunk().await {
                download_len += c.len() as u64;
                content.extend(c);
                on_progress(Progress::Downloading(download_len, content_length));
            }
            on_progress(Progress::Installing());
            let basedirs = BaseDirs::new().expect("Error getting base dirs");
            let sounds_dir = basedirs.home_dir().join(".mlws");
            if !sounds_dir.join("tmp").exists() {
                create_dir_all(sounds_dir.join("tmp")).expect("Error creating tmp folder");
            }
            zip::extract(Cursor::new(content), &sounds_dir.join("tmp"), true)
                .expect("Error extracting archive");
            let sounds: SoundsRON = from_str(
                &read_to_string(sounds_dir.join("tmp").join("sounds.ron"))
                    .expect("Error reading file"),
            )
            .expect("Error parsing sounds");
            if !sounds_dir.join("sounds").exists() {
                create_dir_all(sounds_dir.join("sounds")).expect("Error creating sounds folder");
            }
            rename(
                sounds_dir.join("tmp"),
                sounds_dir.join("sounds").join(&sounds.name),
            )
            .expect("Error renaming repo");
            *data = Some(DownloadedSoundRepo {
                version: latest,
                name: sounds.name,
            });
            on_progress(Progress::Done());
        }
    }
}
