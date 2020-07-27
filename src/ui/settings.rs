use iced::{
    button, executor, text_input, Align, Application, Button, Checkbox, Column, Command, Container,
    Element, Image, Length, Row, Settings, Subscription, Text, TextInput,
};

use log::{error, info};

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    Save,
    Cancel,
    InputFieldChange(String),
    OutputFieldChange(String),
    LoopbackFieldChange(String),
    AutoloopToggle(bool),
}

pub struct SettingsUI {
    soundloop: crate::SoundLoop,
    cancellable: bool,
    config: Config,
    save_state: button::State,
    cancel_state: button::State,
    input_field: TextField,
    output_field: TextField,
    loopback_field: TextField,
}

impl SettingsUI {
    pub fn new(soundloop: crate::SoundLoop) -> Self {
        let cfg = Config::load();
        Self {
            soundloop,
            cancellable: true,
            config: cfg.clone(),
            save_state: Default::default(),
            cancel_state: Default::default(),
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
                self.config.save();
                self.soundloop.restart().expect("Error restarting sound loop");
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
        }
    }

    pub fn view(&mut self, cancellable: bool) -> Element<super::Message> {
        self.cancellable = cancellable;
        let mut bottom = Row::new().align_items(Align::End).push(
            Button::new(&mut self.save_state, Text::new("Save"))
                // .on_press(super::Message::Settings(SettingsMessage::Save))
                .on_press(super::Message::Settings(SettingsMessage::Save)),
        );
        if cancellable {
            bottom = bottom.push(
                Button::new(&mut self.cancel_state, Text::new("Exit"))
                    .on_press(super::Message::Settings(SettingsMessage::Cancel)),
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
                    |x| super::Message::Settings(SettingsMessage::InputFieldChange(x)),
                )
                .padding(10),
            ))
            .push(new_device_setting(
                "Output device",
                TextInput::new(
                    &mut self.output_field.state,
                    "default",
                    self.output_field.value.as_str(),
                    |x| super::Message::Settings(SettingsMessage::OutputFieldChange(x)),
                )
                .padding(10),
            ));
        #[cfg(feature = "autoloop")]
        {
            if !self.config.autoloop {
                devices = devices.push(new_device_setting(
                    "Loopback device (Required)",
                    TextInput::new(
                        &mut self.loopback_field.state,
                        "field required",
                        self.loopback_field.value.as_str(),
                        |x| super::Message::Settings(SettingsMessage::LoopbackFieldChange(x)),
                    )
                    .padding(10),
                ));
            }
            devices = devices.push(Checkbox::new(
                self.config.autoloop,
                "Create loopback automatically",
                |t| super::Message::Settings(SettingsMessage::AutoloopToggle(t)),
            ));
        }
        let c = Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(devices)
            .push(bottom);
        c.into()
    }
}

async fn empty() {}

fn new_device_setting<'a, S, T>(name: S, e: T) -> Element<'a, super::Message>
where
    T: Into<Element<'a, super::Message>>,
    S: ToString,
{
    Row::new()
        .padding(10)
        .push(Container::new(Text::new(name.to_string())).padding(10))
        .push(e)
        .into()
}

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
