use crossbeam_channel::Sender;

use rdev::Key;

use log::error;

use std::collections::HashMap;

use crate::config::Config;

pub struct KeyBindings<Message: Send, F: Fn(String) -> Message> {
    keybinds: Vec<(String, Vec<Key>)>,
    setting_keybind: Option<(usize, Vec<Key>)>,
    hotkeys: crate::hotkey::HotkeyManager,
	m_sender: Sender<Message>,
	on_hotkey: F,
}

impl<Message: Send + 'static, F: Fn(String) -> Message> KeyBindings<Message, F> {
    fn new(m_sender: Sender<Message>, cfg: Config, on_hotkey: F) -> Self {
        let mut r = Self {
            keybinds: Vec::new(),
            setting_keybind: None,
            hotkeys: crate::hotkey::HotkeyManager::new(Default::default()),
			m_sender,
			on_hotkey
        };
        r.load_config(cfg.hotkeys);
        r
    }

    fn load_config(&mut self, hotkeys: HashMap<String, Vec<Key>>) {
        for (name, keys) in hotkeys {
            self.keybinds.push(
                (name, keys)
            );
            self.set(self.keybinds.len() - 1);
        }
    }

    fn save_config(&self) -> HashMap<String, Vec<Key>> {
        let mut hm = HashMap::new();
        for (name, keys) in &self.keybinds {
            hm.insert(name.clone(), keys.clone());
        }
        hm
    }

    fn tick(&mut self) {
        if let Some(d) = self.hotkeys.has_detected() {
            match d {
                crate::hotkey::ThreadMessage::Detected(k) => {
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
       
        self.keybinds[id].1 = keys;
        self.set(id);
        self.setting_keybind = None;
    }

    fn set(&self, id: usize) {
        let (txt, keys) = self.keybinds[id].clone();
        let sender_clone = self.m_sender.clone();
        let (m_send, m_recv) = crossbeam_channel::bounded(1);
        m_send.send((self.on_hotkey)(txt.clone()));
        self.hotkeys.register(
            txt.clone(),
            keys,
            Box::new(move || {
                sender_clone
                    .send(m_recv.recv().unwrap())
                    .expect("Error sending sound message");
            }),
        );
    }

    fn unset(&mut self, id: usize) {
        let txt = self.keybinds[id].0.clone();
        // info!("Unregistering {}", txt);
        self.hotkeys.unregister(txt);
        
    }
}
