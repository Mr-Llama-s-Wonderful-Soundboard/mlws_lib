use std::io::{Cursor, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;

use crossbeam_channel::Sender;
use log::{error, info};
use reqwest::{get, IntoUrl};
use zip_extract as zip;

use crate::utils::bytes_as_str;

pub enum Message {
    Progress(usize),
    End,
}

pub fn download_extract<S: IntoUrl + Clone + Send + 'static>(
    sender: Sender<Message>,
    url_string: S,
    out: PathBuf,
) {
    std::thread::spawn(move || {
        smol::run(async {
            info!(
                "Downloading sounds from {:?}",
                url_string
                    .clone()
                    .into_url()
                    .expect("Error converting into url")
            );
            // let url: Url = url_string.parse().expect("Error parsing URL");

            let mut body = get(url_string).await.expect("Error getting response");
            let mut archive = Vec::new();
            let mut filled = 0;
            let mut failed_attempts = 0;
            loop {
                match body.chunk().await {
                    Ok(maybe_bytes) => {
                        match maybe_bytes {
                            Some(bytes) => {
								//if bytes.len() == 0 {
								//	break;
								//}
								// info!("Filled a chunk");
								bytes.iter().for_each(|b| archive.push(*b));
                                //}
                                info!("Downloading: {}", archive.len());
                                sender
                                    .send(Message::Progress(archive.len()))
                                    .expect("Error sending progress");
                            }
                            None => break,
                        }
                        //if bytes == 0 {
                        // break;
                        //}
                        //filled += bytes;
                        //if filled + 1 >= chunk.len() {
                    }
                    Err(e) => {
                        failed_attempts += 1;
                        error!("[{}] {}", failed_attempts, e);
                        if failed_attempts > 3 {
                            break;
                        }
                    }
                }
			}
			info!("Extracting ZIP");
            zip::extract(Cursor::new(archive), &out, true).expect("Error extracting archive");
            sender.send(Message::End).expect("Error sending end");
        });
    });
}
