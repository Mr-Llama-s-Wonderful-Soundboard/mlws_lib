#[allow(unused)]
use iced::{
    button, scrollable, text_input, Align, Button, Checkbox, Column, Command, Container, Element,
    Length, ProgressBar, Row, Scrollable, Text, TextInput,
};

use directories::BaseDirs;
use log::{error, info};

use rdev::Key;

use super::downloader;
use super::soundboard;
use crate::config::{Config, SoundConfig};
use crossbeam_channel::Sender;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    Save,
    Cancel,
    Update,
    InputFieldChange(String),
    OutputFieldChange(String),
    LoopbackFieldChange(String),
    AutoloopToggle(bool),
    Keybind(KeybindMessage),
}

pub struct SettingsUI {
    soundloop: crate::SoundLoop,
    cancellable: bool,
    config: Config,
    save_state: button::State,
    cancel_state: button::State,
    update_state: button::State,
    input_field: TextField,
    output_field: TextField,
    loopback_field: TextField,
    keybinds: KeyBindings,
}

impl SettingsUI {
    pub fn new(
        soundloop: crate::SoundLoop,
        m_sender: Sender<soundboard::SoundboardMessage>,
    ) -> Self {
        let cfg = Config::load();
        // hotkeys.register(vec![rdev::Key::KeyR], Box::new(move || {
        //     m_sender.send(
        //         soundboard::SoundboardMessage::SoundPressed("Our anthem".into()),
        //     ).expect("Error sending sound request");
        // }));
        Self {
            soundloop,
            cancellable: true,
            config: cfg.clone(),
            save_state: Default::default(),
            cancel_state: Default::default(),
            update_state: Default::default(),
            keybinds: KeyBindings::new(m_sender, cfg.clone()),
            input_field: TextField::new(cfg.input_device.unwrap_or("".into())),
            output_field: TextField::new(cfg.output_device.unwrap_or("".into())),
            loopback_field: TextField::new(cfg.loopback_device.unwrap_or("".into())),
        }
    }

    pub fn update(
        &mut self,
        message: SettingsMessage,
    ) -> (Command<super::Message>, Option<super::AppState>) {
        match message {
            SettingsMessage::Update => {
                let basedirs = BaseDirs::new().expect("Error getting base dirs");
                let sounds_dir = basedirs.home_dir().join(".mlws");

                let (s, r) = crossbeam_channel::unbounded();

                downloader::download_extract(
                    s,
                    "https://github.com/Mr-Llama-s-Wonderful-Soundboard/sounds/archive/master.zip",
                    sounds_dir,
                );
                (
                    Command::perform(empty(), move |_| {
                        loop {
                            if let Ok(downloader::Message::End) = r.recv() {
                                println!("Download done, reloading sounds");
                                break;
                            }
                        }
                        super::Message::Soundboard(
                            super::soundboard::SoundboardMessage::ReloadSounds,
                        )
                    }),
                    None,
                )
            }
            SettingsMessage::Save => {
                info!("SAVING");
                let input = if self.input_field.value == "" {
                    None
                } else {
                    Some(self.input_field.value.clone())
                };
                let output = if self.output_field.value == "" {
                    None
                } else {
                    Some(self.output_field.value.clone())
                };
                let loopback = if self.loopback_field.value == "" {
                    None
                } else {
                    Some(self.loopback_field.value.clone())
                };
                if !self.cancellable {
                    if loopback.is_none() {
                        return (Command::none(), None);
                    }
                }
                self.config.input_device = input;
                self.config.output_device = output;
                self.config.loopback_device = loopback;
                self.config.hotkeys = self.keybinds.save_config();
                self.config.save();
                self.soundloop
                    .restart()
                    .expect("Error restarting sound loop");
                (Command::none(), Some(super::AppState::Soundboard))
            }
            SettingsMessage::Cancel => (Command::none(), Some(super::AppState::Soundboard)),
            SettingsMessage::InputFieldChange(v) => {
                self.input_field.value = v;
                (Command::none(), None)
            }
            SettingsMessage::OutputFieldChange(v) => {
                self.output_field.value = v;
                (Command::none(), None)
            }
            SettingsMessage::LoopbackFieldChange(v) => {
                self.loopback_field.value = v;
                (Command::none(), None)
            }
            SettingsMessage::AutoloopToggle(t) => {
                #[cfg(feature = "autoloop")]
                {
                    self.config.autoloop = t;
                }
                #[cfg(not(feature = "autoloop"))]
                {
                    error!("Autoloop was toggle without the feature")
                }
                (Command::none(), None)
            }
            SettingsMessage::Keybind(k) => {
                self.keybinds.update(k);
                (Command::none(), None)
            }
        }
    }

    pub fn tick(&mut self) {
        self.keybinds.tick();
    }

    pub fn view(&mut self, cancellable: bool) -> Element<SettingsMessage> {
        self.cancellable = cancellable;
        let mut bottom = Row::new()
            .align_items(Align::End)
            .push(
                Button::new(&mut self.update_state, Text::new("UPDATE"))
                    // .on_press(super::Message::Settings(SettingsMessage::Save))
                    .on_press(SettingsMessage::Update),
            )
            .push(
                Button::new(&mut self.save_state, Text::new("Save"))
                    // .on_press(super::Message::Settings(SettingsMessage::Save))
                    .on_press(SettingsMessage::Save),
            );
        if cancellable {
            bottom = bottom.push(
                Button::new(&mut self.cancel_state, Text::new("Exit"))
                    .on_press(SettingsMessage::Cancel),
            )
        }
        #[allow(unused_mut)]
        let mut devices = Column::new()
            .push(new_device_setting(
                "Input device",
                TextInput::new(
                    &mut self.input_field.state,
                    "default",
                    self.input_field.value.as_str(),
                    |x| SettingsMessage::InputFieldChange(x),
                )
                .padding(10),
            ))
            .push(new_device_setting(
                "Output device",
                TextInput::new(
                    &mut self.output_field.state,
                    "default",
                    self.output_field.value.as_str(),
                    |x| SettingsMessage::OutputFieldChange(x),
                )
                .padding(10),
            ));
        #[cfg(not(feature = "autoloop"))]
        {
            devices = devices.push(new_device_setting(
                "Loopback device (Required)",
                TextInput::new(
                    &mut self.loopback_field.state,
                    "field required",
                    self.loopback_field.value.as_str(),
                    |x| SettingsMessage::LoopbackFieldChange(x),
                )
                .padding(10),
            ));
        }

        #[cfg(feature = "autoloop")]
        {
            if !self.config.autoloop {
                devices = devices.push(new_device_setting(
                    "Loopback device (Required)",
                    TextInput::new(
                        &mut self.loopback_field.state,
                        "field required",
                        self.loopback_field.value.as_str(),
                        |x| SettingsMessage::LoopbackFieldChange(x),
                    )
                    .padding(10),
                ));
            }
            devices = devices.push(Checkbox::new(
                self.config.autoloop,
                "Create loopback automatically",
                |t| SettingsMessage::AutoloopToggle(t),
            ));
        }

        let c = Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(devices)
            .push(self.keybinds.view().map(SettingsMessage::Keybind))
            .push(bottom);
        c.into()
    }
}

async fn empty() {}

fn new_device_setting<'a, S, T>(name: S, e: T) -> Element<'a, SettingsMessage>
where
    T: Into<Element<'a, SettingsMessage>>,
    S: ToString,
{
    Row::new()
        .padding(10)
        .push(Container::new(Text::new(name.to_string())).padding(10))
        .push(e)
        .into()
}

#[derive(Debug, Clone)]
pub struct TextField {
    value: String,
    state: text_input::State,
}

impl TextField {
    pub fn new<S>(content: S) -> Self
    where
        S: ToString,
    {
        Self {
            value: content.to_string(),
            state: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum KeybindMessage {
    Add,
    TextChange(usize, String),
    StartKeybind(usize),
    StopKeybind,
    CancelKeybind,
    Reset(usize),
}

pub struct KeyBindings {
    scrollable: scrollable::State,
    add_button: button::State,
    keybinds: Vec<(TextField, Vec<Key>, button::State, button::State)>,
    setting_keybind: Option<(usize, Vec<Key>)>,
    hotkeys: crate::hotkey::HotkeyManager,
    m_sender: Sender<soundboard::SoundboardMessage>,
}

impl KeyBindings {
    fn new(m_sender: Sender<soundboard::SoundboardMessage>, cfg: Config) -> Self {
        let mut r = Self {
            scrollable: Default::default(),
            add_button: Default::default(),
            keybinds: Vec::new(),
            setting_keybind: None,
            hotkeys: crate::hotkey::HotkeyManager::new(Default::default()),
            m_sender,
        };
        r.load_config(cfg.hotkeys);
        r
    }

    fn load_config(&mut self, hotkeys: HashMap<String, Vec<Key>>) {
        for (name, keys) in hotkeys {
            self.keybinds.push((
                TextField::new(name),
                keys,
                Default::default(),
                Default::default(),
            ));
            self.set(self.keybinds.len() - 1);
        }
    }

    fn save_config(&self) -> HashMap<String, Vec<Key>> {
        let mut hm = HashMap::new();
        for (txt, keys, _, _) in &self.keybinds {
            hm.insert(txt.value.clone(), keys.clone());
        }
        hm
    }

    fn update(&mut self, m: KeybindMessage) {
        match m {
            KeybindMessage::Add => self.keybinds.push((
                TextField::new(""),
                vec![Key::Unknown(0)],
                Default::default(),
                Default::default(),
            )),
            KeybindMessage::StartKeybind(id) => {
                self.setting_keybind = Some((id, vec![]));
                self.hotkeys.start_detecting();
            }
            KeybindMessage::StopKeybind => {
                if let Some((id, _)) = self.setting_keybind {
                    let h = self.hotkeys.stop_detecting();
                    self.finished_detecting(id, h);
                }
            }
            KeybindMessage::TextChange(id, v) => {
                self.unset(id);
                self.keybinds[id].0.value = v;
                self.set(id);
            }
            KeybindMessage::Reset(id) => {
                self.unset(id);
                self.keybinds.remove(id);
            }
            KeybindMessage::CancelKeybind => self.setting_keybind = None,
        }
    }

    fn tick(&mut self) {
        if let Some(d) = self.hotkeys.has_detected() {
            match d {
                crate::hotkey::ThreadMessage::Detected(k) => {
                    info!("Detected {:?}", k);
                    let mut old = self.setting_keybind.clone().unwrap();
                    old.1 = k;
                    self.setting_keybind = Some(old);
                }
                crate::hotkey::ThreadMessage::DetectedStopped(_) => {
                    error!("Detect stop should been caught before");
                }
            }
        }
    }

    fn finished_detecting(&mut self, id: usize, keys: Vec<Key>) {
        info!("[{}]: {:?}", id, keys);
        self.keybinds[id].1 = keys;
        self.set(id);
        self.setting_keybind = None;
    }

    fn set(&self, id: usize) {
        let (txt, keys, _, _) = self.keybinds[id].clone();
        let sender_clone = self.m_sender.clone();
        self.hotkeys.register(
            txt.value.clone(),
            keys,
            Box::new(move || {
                sender_clone
                    .send(soundboard::SoundboardMessage::SoundPressed(
                        txt.value.clone(),
                    ))
                    .expect("Error sending sound message");
            }),
        );
    }

    fn unset(&mut self, id: usize) {
        let txt = self.keybinds[id].0.value.clone();
        info!("Unregistering {}", txt);
        self.hotkeys.unregister(txt);
        
    }

    fn view(&mut self) -> Element<KeybindMessage> {
        Column::new()
            .align_items(Align::Center)
            .push(Row::new().push(
                Button::new(&mut self.add_button, Text::new("Add")).on_press(KeybindMessage::Add),
            ))
            .push({
                let mut i = 0;
                let mut scrollable = Scrollable::new(&mut self.scrollable)
                    .align_items(Align::Center)
                    .height(Length::Units(500))
                    .width(Length::Fill);
                for (id, keys, btn_state, reset_btn) in self.keybinds.iter_mut() {
                    scrollable = scrollable.push(
                        Row::new()
                            // .spacing(10)
                            .push(
                                Container::new(TextInput::new(
                                    &mut id.state,
                                    "Please write the sound name",
                                    &id.value.clone(),
                                    move |v| KeybindMessage::TextChange(i, v),
                                ))
                                .align_x(Align::Start)
                                .padding(10)
                                .width(Length::FillPortion(2)),
                            )
                            .push(
                                Container::new(
                                    Row::new()
                                        .push(Text::new(
                                            if let Some((id, k)) = &mut self.setting_keybind {
                                                if i == *id {
                                                    k
                                                } else {
                                                    keys
                                                }
                                            } else {
                                                keys
                                            }
                                            .iter()
                                            .map(|x| x.to_string())
                                            .collect::<Vec<String>>()
                                            .join(" + "),
                                        ))
                                        .push(if let Some((id, _)) = self.setting_keybind {
                                            if i == id {
                                                Button::new(btn_state, Text::new("DONE"))
                                                    .on_press(KeybindMessage::StopKeybind)
                                            } else {
                                                Button::new(btn_state, Text::new("SET"))
                                                    .on_press(KeybindMessage::StartKeybind(i))
                                            }
                                        } else {
                                            Button::new(btn_state, Text::new("SET"))
                                                .on_press(KeybindMessage::StartKeybind(i))
                                        })
                                        .push(if let Some((id, _)) = self.setting_keybind {
                                            if i == id {
                                                Button::new(reset_btn, Text::new("CANCEL"))
                                                    .on_press(KeybindMessage::CancelKeybind)
                                            } else {
                                                Button::new(reset_btn, Text::new("REMOVE"))
                                                    .on_press(KeybindMessage::Reset(i))
                                            }
                                        } else {
                                            Button::new(reset_btn, Text::new("REMOVE"))
                                                .on_press(KeybindMessage::Reset(i))
                                        }),
                                )
                                .align_x(Align::End)
                                .padding(10)
                                .width(Length::FillPortion(2)),
                            )
                            .width(Length::Fill),
                    );
                    i += 1;
                }
                scrollable
            })
            .into()
    }
}
