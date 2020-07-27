use iced::{
    button, executor, Align, Application, Button, Column, Command, Element, Image, Length, Row,
    Settings, Subscription, Text,
};
use iced_native::Event;

use crate::config;
use crate::sound;

mod settings;
mod soundboard;

#[derive(Debug, Clone)]
pub enum Message {
    // SettingsPressed,
    // UpdatePressed,
    // SoundPressed(String),
    Soundboard(soundboard::SoundboardMessage),
    Settings(settings::SettingsMessage),
    Event(Event),
}

#[derive(Debug, Clone, Copy)]
pub enum AppState {
    Settings(bool),
    Soundboard,
}

pub struct App {
    // settings_button: button::State,
    // update_button: button::State,
    // sounds_buttons: Vec<Vec<(String, button::State)>>,
    // sounds: config::SoundConfig,
    // sound_sender: crossbeam_channel::Sender<sound::Message>,
    // sound_reciever: crossbeam_channel::Receiver<sound::Message>,
    soundboard: soundboard::Soundboard,
    settings: settings::SettingsUI,
    state: AppState,
}

impl<'a> Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = (
        crossbeam_channel::Sender<sound::Message>,
        crossbeam_channel::Receiver<sound::Message>,
        AppState,
        crate::SoundLoop,
    );

    fn new(
        (sound_sender, sound_reciever, state, soundloop): Self::Flags,
    ) -> (Self, Command<Message>) {
        let mut r = Self {
            soundboard: soundboard::Soundboard::new(sound_sender, sound_reciever),
            settings: settings::SettingsUI::new(soundloop),
            state,
        };
        r.soundboard.load_sounds();
        (r, Command::none())
    }

    fn title(&self) -> String {
        String::from("Mr Llama's Wonderful Soundboard")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Soundboard(m) => {
				let (cmd, change_state) = self.soundboard.update(m);
                if let Some(state) = change_state {
                    self.state = state;
				}
				cmd
			}
			
            Message::Settings(m) => {
				let (cmd, change_state) = self.settings.update(m);
                if let Some(state) = change_state {
                    self.state = state;
				}
				cmd
            }
            Message::Event(_) => {Command::none()}
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::Event)
    }

    fn view(&mut self) -> Element<Message> {
		
        match self.state {
            AppState::Soundboard => self.soundboard.view(),
            AppState::Settings(cancellable) => self.settings.view(cancellable),
        }
    }
}
