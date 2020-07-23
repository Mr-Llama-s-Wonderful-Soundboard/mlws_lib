use iced::{button, Align, Button, Column, Element, Row, Sandbox, Settings, Text};

use fern;
use log::{error, info, warn};
use recolored::Colorize;

use std::{fs, io};

mod config;
mod pa_ctl;
#[cfg(feature = "autoloop")]
mod pulseauto;
mod sound;
mod sounds_loader;

fn setup_logger() -> Result<(), log::SetLoggerError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}",
                format!("[{}][{}] {}", record.level(), record.target().replace("mr_llamas_wonderful_soundboard", "MLWS"), message).color(
                    match record.level() {
                        log::Level::Warn => "yellow",
                        log::Level::Error => "red",
                        log::Level::Info => "blue",
                        log::Level::Debug => "green",
                        _ => "",
                    }
                ),
            ))
        }) // by default only accept warn messages
        .level(log::LevelFilter::Info)
        .level_for("wgpu_native", log::LevelFilter::Warn)
        .level_for("gfx_backend_vulkan", log::LevelFilter::Warn)
        // accept info messages from the current crate too
        //.level_for("mr_llamas_wonderful_soundboard", log::LevelFilter::Info)
        //.level_for("sound", log::LevelFilter::Info)
        .chain(io::stdout())
        .apply()
}

fn main() {
    //mowl::init_with_level(log::LevelFilter::Info).unwrap();
    //config::SoundsConfig::load();
    setup_logger().unwrap();
    info!("loaded logger");
    config::Config::load();
    App::run(Default::default());
}

#[derive(Default)]
struct App {
    settings_button: button::State,
    update_button: button::State,
    sounds_buttons: Vec<Vec<(String, button::State)>>,
    sounds: config::SoundConfig,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    SettingsPressed,
    UpdatePressed,
    SoundPressed(usize),
}

impl App {
    fn add_sound(&mut self, i: String) {
        let last = self.sounds_buttons.last_mut().unwrap();
        if last.len() > 10 {
            self.sounds_buttons.push(vec![(i, Default::default())])
        } else {
            last.push((i, Default::default()))
        }
    }

    fn load_sounds(&mut self) {
        self.sounds = config::SoundConfig::load();
        self.sounds_buttons.push(vec![]);
        let keys: Vec<String> = self.sounds.sounds.keys().map(|x| x.clone()).collect();
        for i in keys {
            self.add_sound(i.clone());
        }
    }
}

impl<'a> Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        let mut r = Self::default();
        r.load_sounds();
        r
    }

    fn title(&self) -> String {
        String::from("Mr Llama's Wonderful Soundboard")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SettingsPressed => {}
            Message::SoundPressed(id) => info!("TODO PLAY SOUND {}", id),
            Message::UpdatePressed => {
                config::SoundConfig::update();
                std::process::exit(0);
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let mut c = Column::new().padding(20).align_items(Align::Center);
        let mut i = 0;
        c = self.sounds_buttons.iter_mut().fold(
            c.align_items(Align::Center),
            |column, row_items| {
                let row = row_items.iter_mut().fold(
                    Row::new().padding(20).align_items(Align::Center),
                    |row: Row<Message>, item| {
                        let b = Button::new(&mut item.1, Text::new(item.0.clone()))
                            .on_press(Message::SoundPressed(i));
                        i += 1;
                        row.push(b)
                    },
                );
                column.push(row)
            },
        );

        c.push(
            Button::new(&mut self.settings_button, Text::new("⚙"))
                .on_press(Message::SettingsPressed),
        )
        .push(
            Button::new(&mut self.update_button, Text::new("UPDATE"))
                .on_press(Message::UpdatePressed),
        )
        .into()
    }
}
