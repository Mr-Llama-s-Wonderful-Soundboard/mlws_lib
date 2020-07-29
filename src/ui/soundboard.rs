use iced::{
    button, executor, Align, Application, Button, Column, Command, Element, Image, Length, Row,
    Settings, Subscription, Text,
};

use log::info;

use crate::config;
use crate::sound;

#[derive(Debug, Clone)]
pub enum SoundboardMessage {
    SettingsPressed,
	SoundPressed(String),
	ReloadSounds,
}

pub struct Soundboard {
    settings_button: button::State,
    update_button: button::State,
    sounds_buttons: Vec<Vec<(String, button::State)>>,
    sounds: config::SoundConfig,
    sound_sender: crossbeam_channel::Sender<sound::Message>,
    sound_reciever: crossbeam_channel::Receiver<sound::Message>,
}

impl Soundboard {
    pub fn new(
        sound_sender: crossbeam_channel::Sender<sound::Message>,
        sound_reciever: crossbeam_channel::Receiver<sound::Message>,
    ) -> Self {
        let mut r = Self {
            sound_sender,
            sound_reciever,
            settings_button: Default::default(),
            update_button: Default::default(),
            sounds_buttons: Default::default(),
            sounds: Default::default(),
        };
        r.load_sounds();
        r
    }

    pub fn update(&mut self, message: SoundboardMessage) -> (Command<super::Message>, Option<super::AppState>) {
        match message {
            SoundboardMessage::SettingsPressed => (Command::none(), Some(super::AppState::Settings(true))),
            SoundboardMessage::SoundPressed(id) => {
                let sound = self.sounds.sounds.get(&id).unwrap().clone();
                self.sound_sender
                    .send(sound::Message::PlaySound(sound, sound::SoundDevices::Both))
                    .expect("Error sending play sound");
					(Command::none(), None)
			}
			SoundboardMessage::ReloadSounds => {
				self.load_sounds();
				(Command::none(), None)
			}
        }
    }

    pub fn view(&mut self) -> Element<super::Message> {
        let mut c = Column::new().padding(20).align_items(Align::Center);
        let mut i = 0;
        let sounds_clone = self.sounds.clone();
        c = self.sounds_buttons.iter_mut().fold(
            c.align_items(Align::Center),
            |column, row_items| {
                let row = row_items.iter_mut().fold(
                    Row::new().padding(20).align_items(Align::Center),
                    |row: Row<super::Message>, item| {
                        let msg = SoundboardMessage::SoundPressed(item.0.clone());
                        let sound = sounds_clone.get(&item.0).unwrap().clone();
                        let img = Image::new(sound.img.unwrap()).width(Length::Units(100));
                        let b = Button::new(&mut item.1, Text::new(item.0.clone()))
                            .on_press(super::Message::Soundboard(msg));
                        let item_column = Column::new()
                            .padding(20)
                            .align_items(Align::Center)
                            .push(img)
                            .push(b);
                        i += 1;
                        row.push(item_column)
                    },
                );
                column.push(row)
            },
        );
        c.push(
                Button::new(&mut self.settings_button, Text::new("⚙"))
                    .on_press(super::Message::Soundboard(SoundboardMessage::SettingsPressed)),
            )
            .into()
    }

    fn add_sound(&mut self, i: String) {
        let last = self.sounds_buttons.last_mut().unwrap();
        if last.len() >= 5 {
            self.sounds_buttons.push(vec![(i, Default::default())])
        } else {
            last.push((i, Default::default()))
        }
    }

    pub fn load_sounds(&mut self) {
        info!("LOADING SOUNDS");
        self.sounds = config::SoundConfig::load();
        info!("{:?}", self.sounds);
        self.update_sounds();
    }

    pub fn update_sounds(&mut self) {
        self.sounds_buttons = Default::default();
        self.sounds_buttons.push(vec![]);
        let keys: Vec<String> = self.sounds.sounds.keys().map(|x| x.clone()).collect();
        for i in keys {
            self.add_sound(i.clone());
        }
    }
}
