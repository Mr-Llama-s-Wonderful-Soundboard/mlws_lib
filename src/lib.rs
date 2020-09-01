pub mod config;
#[cfg(feature = "autoloop")]
mod pulseauto;
#[cfg(feature = "autoloop")]
use pulseauto::Module as PaModule;
#[allow(unused)]
#[cfg(not(feature = "autoloop"))]
struct PaModule; // So that optionals for dropping work
pub mod sound;

pub mod hotkey;
pub mod keybind;



pub fn setup() -> (crossbeam_channel::Sender<sound::Message>, crossbeam_channel::Receiver<sound::Message>, SoundLoop) {
	#[allow(unused)]
    let mut conf = config::Config::load();

    #[cfg(feature = "autoloop")]
    if conf.autoloop {
        info!("Loading modules");
        let _null_sink = match PaModule::load(
            "module-null-sink",
            "sink_name=SoundboardNullSink sink_properties=device.description=SoundboardNullSink",
        ) {
            Ok(r) => {
                conf.loopback_device = Some("SoundboardNullSink".into());
                conf.save();
                r
            }
            Err(e) => panic!("Error: {}", e),
        };

        //_maybe_null_sink = Some(null_sink);

        let _loopback = match PaModule::load(
            "module-loopback",
            "source=@DEFAULT_SOURCE@ sink=SoundboardNullSink latency_msec=5",
        ) {
            Ok(r) => r,
            Err(e) => panic!("Error: {}", e),
        };
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

    #[allow(unused_mut)]
	let soundloop = SoundLoop::new(gui_sender_clone, sound_receiver, sound_sender);
	(gui_sender, gui_receiver, soundloop)
}

pub struct SoundLoop {
    output_id: Option<String>,
    input_id: Option<String>,
    loopback_id: Option<String>,

    gui_sender: crossbeam_channel::Sender<sound::Message>,
    sound_receiver: crossbeam_channel::Receiver<sound::Message>,
    sound_sender: crossbeam_channel::Sender<sound::Message>,

    thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl SoundLoop {
    pub fn new(
        gui_sender: crossbeam_channel::Sender<sound::Message>,
        sound_receiver: crossbeam_channel::Receiver<sound::Message>,
        sound_sender: crossbeam_channel::Sender<sound::Message>,
    ) -> Self {
        let mut r = Self {
            gui_sender,
            sound_receiver,
            sound_sender,
            output_id: None,
            input_id: None,
            loopback_id: None,
            thread_handle: None,
        };
        r.load();
        r
    }

    pub fn load(&mut self) {
        let conf = config::Config::load();
        self.loopback_id = conf.loopback_device;
        self.output_id = conf.output_device;
        self.input_id = conf.input_device;
    }

    pub fn run(&mut self) -> Result<(), ()> {
        if let Some(loopback_id) = self.loopback_id.clone() {
            //  START AUDIO THREAD
            let sound_receiver = self.sound_receiver.clone();
            let sound_sender = self.sound_sender.clone();
            let gui_sender = self.gui_sender.clone();

            let input_id = self.input_id.clone();
            let output_id = self.output_id.clone();

            self.thread_handle = Some(std::thread::spawn(move || {
                sound::run_sound_loop(
                    sound_receiver,
                    sound_sender,
                    gui_sender,
                    input_id,
                    output_id,
                    loopback_id,
                );
            }));

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn stop(&mut self) {
        if self.thread_handle.is_some() {
            self.gui_sender
                .send(sound::Message::Kill)
                .expect("Error sending kill signal");
            self.thread_handle = None;
        }
    }

    pub fn restart(&mut self) -> Result<(), ()> {
        self.stop();
        self.load();
        self.run()
    }
}