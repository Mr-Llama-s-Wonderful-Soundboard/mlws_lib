use iced::{
    button, executor, Align, Application, Button, Column, Command, Element, Image, Length, Row,
    Settings, Subscription, Text,
};
use iced_native::Event;
use async_std;
use std::time::{Duration, Instant};

use log::info;

use crate::config;
use crate::sound;

mod downloader;
mod settings;
mod soundboard;

#[derive(Debug, Clone)]
pub enum Message {
    // SettingsPressed,
    // UpdatePressed,
    // SoundPressed(String),
    Soundboard(soundboard::SoundboardMessage),
    Settings(settings::SettingsMessage),
    Tick
    //Event(Event),
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
    hotkeys: crate::hotkey::HotkeyManager,
    message_receiver: crossbeam_channel::Receiver<soundboard::SoundboardMessage>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = (
        crossbeam_channel::Sender<sound::Message>,
        crossbeam_channel::Receiver<sound::Message>,
        AppState,
        crate::SoundLoop,
        crate::hotkey::HotkeyManager,
    );

    fn new(
        (sound_sender, sound_reciever, state, soundloop, hotkeys): Self::Flags,
    ) -> (Self, Command<Message>) {
        let (m_sender, m_receiver) = crossbeam_channel::unbounded();
        let mut r = Self {
            soundboard: soundboard::Soundboard::new(sound_sender, sound_reciever),
            settings: settings::SettingsUI::new(soundloop),
            state,
            hotkeys,
            message_receiver: m_receiver,
        };
        r.soundboard.load_sounds();
        r.hotkeys.register(vec![rdev::Key::KeyR], Box::new(move || {
            m_sender.send(
                soundboard::SoundboardMessage::SoundPressed("Our anthem".into()),
            ).expect("Error sending sound request");
        }));
        (r, Command::none())
    }

    fn title(&self) -> String {
        String::from("Mr Llama's Wonderful Soundboard")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        if let Ok(m) = self.message_receiver.try_recv() {
            self.soundboard.update(m);
        }
        match message {
            Message::Soundboard(m) => {
                let (cmd, change_state) = self.soundboard.update(m);
                if let Some(state) = change_state {
                    info!("Started detecting");
                    self.hotkeys.start_detecting();
                    self.state = state;
                }
                cmd
            }

            Message::Settings(m) => {
                let (cmd, change_state) = self.settings.update(m);
                if let Some(state) = change_state {
                    info!("RESULT KEYS: {:?}", self.hotkeys.stop_detecting());
                    self.state = state;
                }
                cmd
            }
            Message::Tick => {Command::none()} //Message::Event(_) => {Command::none()}
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(10)).map(|_|Message::Tick)
    }

    fn view(&mut self) -> Element<Message> {
        match self.state {
            AppState::Soundboard => self.soundboard.view(),
            AppState::Settings(cancellable) => self.settings.view(cancellable),
        }
    }
}

mod time {
    use iced::futures;

    pub fn every(
        duration: std::time::Duration,
    ) -> iced::Subscription<std::time::Instant> {
        iced::Subscription::from_recipe(Every(duration))
    }

    struct Every(std::time::Duration);

    impl<H, I> iced_native::subscription::Recipe<H, I> for Every
    where
        H: std::hash::Hasher,
    {
        type Output = std::time::Instant;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.0.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            async_std::stream::interval(self.0)
                .map(|_| std::time::Instant::now())
                .boxed()
        }
    }
}
