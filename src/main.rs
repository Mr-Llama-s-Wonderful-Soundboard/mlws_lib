use iced::{
    button, executor, Align, Application, Button, Column, Command, Element, Row, Settings,
    Subscription, Text, Image, Length
};
use iced_native::Event;

use fern;
use log::{error, info, warn};
use recolored::Colorize;

#[cfg(feature = "autoloop")]
use ctrlc;

use std::{fs, io};

mod config;
mod pa_ctl;
#[cfg(feature = "autoloop")]
mod pulseauto;
#[cfg(feature = "autoloop")]
use pulseauto::Module as PaModule;
#[cfg(not(feature = "autoloop"))]
struct PaModule; // So that optionals for dropping work
mod sound;
mod sounds_loader;

fn setup_logger() -> Result<(), log::SetLoggerError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}",
                format!(
                    "[{}][{}] {}",
                    record.level(),
                    record
                        .target()
                        .replace("mr_llamas_wonderful_soundboard", "MLWS"),
                    message
                )
                .color(match record.level() {
                    log::Level::Warn => "yellow",
                    log::Level::Error => "red",
                    log::Level::Info => "blue",
                    log::Level::Debug => "green",
                    _ => "",
                }),
            ))
        }) // by default only accept warn messages
        .level(log::LevelFilter::Info)
        //.level_for("mr_llamas_wonderful_soundboard::pulseauto", log::LevelFilter::Debug)
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
    let conf = config::Config::load();

    #[allow(unused_mut)]
    let mut loopback_id = conf.loopback_device;
    let output_id = conf.output_device;
    let input_id = conf.input_device;

    #[cfg(feature = "autoloop")]
    {
        info!("Loading modules");
        let null_sink = match PaModule::load(
            "module-null-sink",
            "sink_name=SoundboardNullSink sink_properties=device.description=SoundboardNullSink",
        ) {
            Ok(r) => {
                loopback_id = Some("SoundboardNullSink".into());
                r
            }
            Err(e) => panic!("Error: {}", e),
        };

        //_maybe_null_sink = Some(null_sink);

        let loopback = match PaModule::load(
            "module-loopback",
            "source=@DEFAULT_SOURCE@ sink=SoundboardNullSink latency_msec=5",
        ) {
            Ok(r) => r,
            Err(e) => panic!("Error: {}", e),
        };

        //_maybe_loopback = Some(loopback);

        // ctrlc::set_handler(move || {
        //     loopback.unload();
        //     null_sink.unload();
        //     std::process::exit(0)
        // })
        // .expect("Error setting up ctrlc handler");
    }

    let (sound_sender, gui_receiver): (
        crossbeam_channel::Sender<sound::Message>,
        crossbeam_channel::Receiver<sound::Message>,
    ) = crossbeam_channel::unbounded();

    let (gui_sender, sound_receiver): (
        crossbeam_channel::Sender<sound::Message>,
        crossbeam_channel::Receiver<sound::Message>,
    ) = crossbeam_channel::unbounded();

    let gui_sender_clone = gui_sender.clone();

    let mut state = AppState::Settings(true);
    if let Some(loopback_id) = loopback_id {
        state = AppState::Soundboard;
        //  START AUDIO THREAD
        let _sound_thread_handle = std::thread::spawn(move || {
            sound::run_sound_loop(
                sound_receiver,
                sound_sender,
                gui_sender_clone,
                input_id,
                output_id,
                loopback_id,
            );
        });
    }

    let mut settings = Settings::with_flags((gui_sender, gui_receiver, state));
    settings.window.decorations = true;
    App::run(settings);
}

#[derive(Debug, Clone)]
enum Message {
    SettingsPressed,
    UpdatePressed,
    SoundPressed(String),
    Event(Event),
}

#[derive(Debug, Clone, Copy)]
enum AppState {
    Settings(bool),
    Soundboard,
}

struct App {
    settings_button: button::State,
    update_button: button::State,
    sounds_buttons: Vec<Vec<(String, button::State)>>,
    sounds: config::SoundConfig,
    sound_sender: crossbeam_channel::Sender<sound::Message>,
    sound_reciever: crossbeam_channel::Receiver<sound::Message>,
    state: AppState,
}

impl App {
    fn add_sound(&mut self, i: String) {
        let last = self.sounds_buttons.last_mut().unwrap();
        if last.len() >= 5 {
            self.sounds_buttons.push(vec![(i, Default::default())])
        } else {
            last.push((i, Default::default()))
        }
    }

    fn load_sounds(&mut self) {
        self.sounds = config::SoundConfig::load();
        self.update_sounds();
    }

    fn update_sounds(&mut self) {
        self.sounds_buttons = Default::default();
        self.sounds_buttons.push(vec![]);
        let keys: Vec<String> = self.sounds.sounds.keys().map(|x| x.clone()).collect();
        for i in keys {
            self.add_sound(i.clone());
        }
    }
}

impl<'a> Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = (
        crossbeam_channel::Sender<sound::Message>,
        crossbeam_channel::Receiver<sound::Message>,
        AppState,
    );

    fn new((sound_sender, sound_reciever, state): Self::Flags) -> (Self, Command<Message>) {
        let mut r = Self {
            sound_sender,
            sound_reciever,
            settings_button: Default::default(),
            update_button: Default::default(),
            sounds_buttons: Default::default(),
            sounds: Default::default(),
            state,
        };
        r.load_sounds();
        (r, Command::none())
    }

    fn title(&self) -> String {
        String::from("Mr Llama's Wonderful Soundboard")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SettingsPressed => {
                self.state = AppState::Settings(false)
            }
            Message::SoundPressed(id) => {
                let sound = self.sounds.sounds.get(&id).unwrap().clone();
                self.sound_sender
                    .send(sound::Message::PlaySound(sound, sound::SoundDevices::Both))
                    .expect("Error sending play sound");
            }
            Message::UpdatePressed => {
                self.sounds = config::SoundConfig::update();
                self.update_sounds();
                println!("{:?}", self.sounds);
            }
            Message::Event(_) => {}
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::Event)
    }

    fn view(&mut self) -> Element<Message> {
        match self.state {
            AppState::Soundboard => {
                let mut c = Column::new().padding(20).align_items(Align::Center);
                let mut i = 0;
                let sounds_clone = self.sounds.clone();
                c = self.sounds_buttons.iter_mut().fold(
                    c.align_items(Align::Center),
                    |column, row_items| {
                        let row = row_items.iter_mut().fold(
                            Row::new().padding(20).align_items(Align::Center),
                            |row: Row<Message>, item| {
                                let msg = Message::SoundPressed(item.0.clone());
                                let sound = sounds_clone.get(&item.0).unwrap().clone();
                                let img = Image::new(sound.img.unwrap()).width(Length::Units(100));
                                let b = Button::new(&mut item.1, Text::new(item.0.clone()))
                                    .on_press(msg);
                                let item_column = Column::new().padding(20).align_items(Align::Center).push(img).push(b);
                                i += 1;
                                row.push(item_column)
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
            AppState::Settings(_) => todo!(),
        }
    }
}
