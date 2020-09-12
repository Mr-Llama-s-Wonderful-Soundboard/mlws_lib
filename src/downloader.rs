use crate::config::{DownloadedSoundRepo, SoundRepo, SoundsRON};

use std::fs::{create_dir_all, read_to_string, remove_dir_all, rename};
use std::io::Cursor;

use directories::BaseDirs;
use reqwest;
use zip_extract as zip;

use ron::from_str;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Latest(String),
    Updatable(Option<String>, String),
}

#[derive(Debug, Serialize, Deserialize)]
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
    println!("REPO STATUS: {}", latest_version);
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
                println!("DOWNLADING");
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
            if sounds_dir.join("sounds").join(&sounds.name).exists() {
                remove_dir_all(sounds_dir.join("sounds").join(&sounds.name)).unwrap();
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

pub async fn download_async<F: Fn(Progress) -> Fut, Fut: smol::future::Future>(
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
            smol::future::yield_now().await;
            while let Ok(Some(c)) = response.chunk().await {
                println!("DOWNLADING");
                download_len += c.len() as u64;
                content.extend(c);
                on_progress(Progress::Downloading(download_len, content_length)).await;
                smol::future::yield_now().await;
            }
            on_progress(Progress::Installing());
            smol::future::yield_now().await;
            let basedirs = BaseDirs::new().expect("Error getting base dirs");
            let sounds_dir = basedirs.home_dir().join(".mlws");
            if !sounds_dir.join("tmp").exists() {
                create_dir_all(sounds_dir.join("tmp")).expect("Error creating tmp folder");
            }
            zip::extract(Cursor::new(content), &sounds_dir.join("tmp"), true)
                .expect("Error extracting archive");
            smol::future::yield_now().await;
            let sounds: SoundsRON = from_str(
                &read_to_string(sounds_dir.join("tmp").join("sounds.ron"))
                    .expect("Error reading file"),
            )
            .expect("Error parsing sounds");
            if !sounds_dir.join("sounds").exists() {
                create_dir_all(sounds_dir.join("sounds")).expect("Error creating sounds folder");
            }
            if sounds_dir.join("sounds").join(&sounds.name).exists() {
                remove_dir_all(sounds_dir.join("sounds").join(&sounds.name)).unwrap();
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

use smol;

pub fn download_threaded<F: Fn(Progress) + Send + Clone + 'static>(
    repo: SoundRepo,
    mut data: Option<DownloadedSoundRepo>,
    on_progress: F,
    s: std::sync::mpsc::Sender<Option<DownloadedSoundRepo>>,
) {
    std::thread::Builder::new().name("Downloader".into()).spawn(move || {
        smol::block_on(async{
            let status = smol::block_on(status(&repo, &mut data));
            match status {
                Status::Latest(_) => (), // Nothing to do,
                Status::Updatable(_, latest) => {
                    let mut response =
                        reqwest::get(&repo.zip_url).await.expect("Error getting repo zip");
                    let content_length = response.content_length();
                    let on_progress_clone = on_progress.clone();
                    std::thread::spawn(move ||on_progress_clone(Progress::Downloading(0, content_length)));
                    
                    let mut content = Vec::new();
                    let mut download_len = 0;
                    println!("LENGTH: {:?}", content_length);
                    while content_length.map(|l| l > download_len).unwrap_or(true) {
                        println!("TRYING TO DOWNLOAD");
                        if let Some(c) = response.chunk().await.expect("Error getting chunk")
                        {
                            println!("DOWNLADING");
                            download_len += c.len() as u64;
                            content.extend(c);
                            let on_progress_clone = on_progress.clone();
                            std::thread::spawn(move ||on_progress_clone(Progress::Downloading(download_len, content_length)));
                        } else if let None = content_length {
                            println!("BREAKING");
                            break;
                        }
                    }
                    if let Some(l) = content_length {
                        assert!(l == download_len);
                    }

                    let on_progress_clone = on_progress.clone();
                    std::thread::spawn(move ||on_progress_clone(Progress::Installing()));
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
                        create_dir_all(sounds_dir.join("sounds"))
                            .expect("Error creating sounds folder");
                    }
                    if sounds_dir.join("sounds").join(&sounds.name).exists() {
                        remove_dir_all(sounds_dir.join("sounds").join(&sounds.name)).unwrap();
                    }
                    rename(
                        sounds_dir.join("tmp"),
                        sounds_dir.join("sounds").join(&sounds.name),
                    )
                    .expect("Error renaming repo");
                    data = Some(DownloadedSoundRepo {
                        version: latest,
                        name: sounds.name,
                    });
                    std::thread::spawn(move ||on_progress(Progress::Done()));
                    s.send(data).unwrap();
                }
            }
        });
        
    }).unwrap();
}
