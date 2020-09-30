use crossbeam_channel::Sender;

use rdev::Key;

use log::error;

use std::collections::HashMap;
use std::hash::Hash;

use crate::config::Config;

pub struct KeyBindings<Message: Send + 'static, F: Fn(K) -> Message + Send, K: Eq + Hash + Clone + Send> {
    keybinds: Vec<(K, Vec<Key>)>,
    setting_keybind: Option<(usize, Vec<Key>)>,
    hotkeys: crate::hotkey::HotkeyManager<K>,
    m_sender: Sender<Message>,
    on_hotkey: F,
}

impl<Message, F> KeyBindings<Message, F, (String, String)>
where
    Message: Send + 'static,
    F: Fn((String, String)) -> Message + Send,
{
    pub fn new(m_sender: Sender<Message>, cfg: Config, on_hotkey: F) -> Self
    where
        Message: Clone,
    {
        let mut r = Self {
            keybinds: Vec::new(),
            setting_keybind: None,
            hotkeys: crate::hotkey::HotkeyManager::new(Default::default()),
            m_sender,
            on_hotkey,
        };
        r.load_config(cfg.hotkeys);
        r
    }

    pub fn load_config(&mut self, hotkeys: HashMap<(String, String), Vec<Key>>)
    where
        Message: Clone,
    {
        self.remove_everything();
        for (name, keys) in hotkeys {
            self.keybinds.push((name, keys));
            self.set(self.keybinds.len() - 1);
        }
    }

    pub fn add(&mut self, sound: (String, String), keys: Vec<Key>)
    where
        Message: Clone,
    {
        self.keybinds.push((sound, keys));
        self.set(self.keybinds.len() - 1);
    }

    pub fn save_config(&self) -> HashMap<(String, String), Vec<Key>> {
        let mut hm = HashMap::new();
        for (name, keys) in &self.keybinds {
            hm.insert(name.clone(), keys.clone());
        }
        hm
    }

    pub fn tick(&mut self) {
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

    pub fn start_detecting(&mut self, id: usize) {
        self.setting_keybind = Some((id, vec![]));
        self.hotkeys.start_detecting();
    }

    pub fn stop_detecting(&mut self) where Message: Clone {
        if let Some((id, _)) = self.setting_keybind {
            let h = self.hotkeys.stop_detecting();
            self.finished_detecting(id, h);
        }
    }

    pub fn has_detected(&self) -> Option<Vec<Key>> {
		match self.hotkeys.has_detected() {
            Some(crate::hotkey::ThreadMessage::Detected(k)) => Some(k),
            _ => None
        }
	}

    pub fn finished_detecting(&mut self, id: usize, keys: Vec<Key>)
    where
        Message: Clone,
    {
        self.keybinds[id].1 = keys;
        self.set(id);
        self.setting_keybind = None;
    }

    pub fn is_detecting(&self) -> bool {
        self.setting_keybind.is_some()
    }

    pub fn set(&self, id: usize)
    where
        Message: Clone,
    {
        let (txt, keys) = self.keybinds[id].clone();
        let sender_clone = self.m_sender.clone();
        let (m_send, m_recv) = crossbeam_channel::bounded(1);
        m_send.send((self.on_hotkey)(txt.clone())).unwrap();
        let s = m_send.clone();

        self.hotkeys.register(
            txt.clone(),
            keys,
            Box::new(move || {
                let m = m_recv.recv().unwrap();
                sender_clone
                    .send(m.clone())
                    .expect("Error sending sound message");
                s.send(m).unwrap();
            }),
        );
    }

    pub fn unset(&mut self, id: usize) {
        let txt = self.keybinds[id].0.clone();
        // info!("Unregistering {}", txt);
        self.hotkeys.unregister(txt);
    }

    pub fn remove_everything(&mut self) {
        for i in 0..self.keybinds.len() {
            self.unset(i);
        }
        self.keybinds = vec![];
    }

    pub fn keys(&self) -> Vec<((String, String), Vec<Key>)> {
        self.keybinds.clone()
    }
}
