use rdev::{listen, Event, Key, EventType};
use crossbeam_channel::{unbounded, Sender, Receiver};
use lazy_static::lazy_static;

use std::collections::{HashMap};

#[allow(unused)]
use log::{warn, info, error};

lazy_static!{
	static ref EVENT_CHANNEL: (Sender<Event>, Receiver<Event>) = unbounded();
	static ref EVENT_SENDER: Sender<Event> = EVENT_CHANNEL.0.clone();
}

#[allow(unused)]
fn send_event(e: Event) {
	let sender = &EVENT_SENDER;
	sender.send(e).expect("Error sending event");
	//info!("EVENT SENT");
}
/// For sending from the manager to the thread
pub enum ManagerMessage {
	StartDetect,
	StopDetect,
	Stop,
	Loopback(ThreadMessage),
	Register(String, Vec<Key>, Box<dyn Fn() + Send>),
	Unregister(String)
}

/// For sending from the thread to the manager
pub enum ThreadMessage {
	Detected(Vec<Key>),
	DetectedStopped(Vec<Key>),
}

pub struct HotkeyManager {
	/// For sending from the manager to the thread
	manager_sender: Sender<ManagerMessage>,

	/// For sending from the thread to the manager
	pub manager_receiver: Receiver<ThreadMessage>
}

impl HotkeyManager {
	pub fn new(hotkeys: HashMap<String, (Vec<Key>, Box<dyn Fn() + Send>)>) -> Self {
		let (manager_sender, thread_receiver) = unbounded();
		let (thread_sender, manager_receiver) = unbounded();
		
		std::thread::spawn(move ||{
			info!("Listening for keys");
			listen(send_event).expect("Error listening")
		});
		std::thread::spawn(move ||{
			info!("Listening for keys (CHANNEL THREAD)");
			let mut detected: Option<Vec<Key>> = None;
			let mut hotkeys: HashMap<String, (Vec<Key>, Box<dyn Fn() + Send>)> = hotkeys;
			let mut pressed_keys: Vec<Key> = Vec::new();
			loop {
				//info!("Trying to recv");
				match EVENT_CHANNEL.1.try_recv() {
					Ok(event) => {
						match event.event_type {
							EventType::KeyPress(k) => {
								
								if let Some(v) = &mut detected {
									//warn!("DETECTED {:?}", k);
									if !v.contains(&k) {
										v.push(k);
										thread_sender.send(ThreadMessage::Detected(v.clone())).expect("Error sending update");
									}
								}
								pressed_keys.push(k);
								//warn!("P {:?}", pressed_keys);
								for (_, (key, f)) in hotkeys.iter() {
									if key.iter().fold(true, |v, x| v&&pressed_keys.contains(x)) {
										pressed_keys.retain(|x| x != &k);
										f();
									}
								}
							}
							EventType::KeyRelease(k) => {
								pressed_keys.retain(|x| x != &k);
								//warn!("R {:?}", pressed_keys);
							}
							_ => {}
						};
					}
					Err(_) => {}//warn!("RecvErr: {:?}", e)
				};
				if let Ok(message) = thread_receiver.try_recv() {
					match message {
						ManagerMessage::StartDetect => {
							detected = Some(Vec::new());
						}
						ManagerMessage::StopDetect => {
							let v = detected.expect("Detected keys wasn't initialized");
							//warn!("KEYS: {:?}", v);
							thread_sender.send(ThreadMessage::DetectedStopped(v)).expect("Error sending detected keys");
							detected = None;
						}
						ManagerMessage::Stop => {break;}
						ManagerMessage::Loopback(m) => {thread_sender.send(m).expect("Error looping back message");}
						ManagerMessage::Register(name, keys, f) => {
							hotkeys.insert(name,(keys, f));
							info!("{:?}", hotkeys.keys());
						}
						ManagerMessage::Unregister(keys) => {
							info!("Unregistering {}", keys);
							info!("{:?}", hotkeys.keys());
							hotkeys.remove(&keys);
							info!("{:?}", hotkeys.keys());
						}

					};
				}
				
			}
		});
		Self {
			manager_sender, manager_receiver
		}
	}

	pub fn start_detecting(&self) {
		self.manager_sender.send(ManagerMessage::StartDetect).expect("Error sending message");
	}

	pub fn has_detected(&self) -> Option<ThreadMessage> {
		self.manager_receiver.try_recv().ok()
	}

	pub fn stop_detecting(&self) -> Vec<Key> {
		self.manager_sender.send(ManagerMessage::StopDetect).expect("Error sending message");
		loop {
			if let Ok(m) =  self.manager_receiver.recv() {
				match m {
					ThreadMessage::DetectedStopped(v) => {return v;}
					ThreadMessage::Detected(_) => {} // Discard
					// loopback => {self.manager_sender.send(ManagerMessage::Loopback(loopback)).expect("Error sending loopback")}
				}
			}
		}
	}

	pub fn stop(&self) {
		self.manager_sender.send(ManagerMessage::Stop).expect("Error sending stop signal");
	}

	pub fn register(&self,name: String, keys: Vec<Key>, f: Box<dyn Fn() + Send>) {
		self.manager_sender.send(ManagerMessage::Register(name, keys, f)).expect("Error sending register request");
	}

	pub fn unregister(&self,name: String) {
		self.manager_sender.send(ManagerMessage::Unregister(name)).expect("Error sending unregister request");
	}
}
