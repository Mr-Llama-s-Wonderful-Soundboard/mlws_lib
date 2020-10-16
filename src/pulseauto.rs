use anyhow::{anyhow, Result};
use libpulse_binding as pulse;
use log::{debug, info, trace, warn};
use pulse::context::State;

type ListResult<T> = pulse::callbacks::ListResult<T>;

#[allow(dead_code)]
pub struct Module {
    id: u32,
}

impl Module {
    pub fn load(module_name: &str, args: &str) -> Result<Self> {
        info!("Loading module {} {})", module_name, args);
        let id = load_module(module_name, args)?;
        info!("Loaded module #{} ({} {})", id, module_name, args);
        Ok(Self { id })
    }

    #[allow(dead_code)]
    pub fn unload(&self) -> Result<()> {
        info!("Unloading module {}", self.id);
        unload_module(self.id)
    }
}

// impl Drop for Module {
//     fn drop(&mut self) {
//         self.unload().expect("Error unloading module")
//     }
// }

pub fn load_module(module_name: &str, args: &str) -> Result<u32> {
    let (mut mainloop, pulse_context) = connect_pulse()?;

    mainloop.lock();

    let mut introspector = pulse_context.introspect();
    let (check_sender, check_receiver): (
        crossbeam_channel::Sender<ListResult<(bool, u32)>>,
        crossbeam_channel::Receiver<ListResult<(bool, u32)>>,
    ) = crossbeam_channel::unbounded();
    let name_clone = String::from(module_name);
    let args_clone = String::from(args);
    let check_module_callback = move |module_info: pulse::callbacks::ListResult<
        &pulse::context::introspect::ModuleInfo,
    >| {
        //debug!("Gotten list result");
        let r = match module_info {
            ListResult::Error => ListResult::Error,
            ListResult::End => ListResult::Error,
            ListResult::Item(i) => {
                debug!(
                    "{}: {} {}",
                    i.index,
                    i.name.clone().unwrap_or("<UNNAMED>".into()),
                    i.argument.clone().unwrap_or("<NO ARGS>".into())
                );
                pulse::callbacks::ListResult::Item((
                    i.name
                        .clone()
                        .and_then(|name| Some(name == name_clone))
                        .unwrap_or_default()
                        && i.argument
                            .clone()
                            .and_then(|a| Some(a == args_clone))
                            .unwrap_or_default(),
                    i.index,
                ))
            }
        };
        if let Err(e) = check_sender.send(r) {
            println!("Pulse sender_error: {}", e.to_string());
        }
    };
    introspector.get_module_info_list(check_module_callback);
    mainloop.unlock();
    let mut present = None;
    loop {
        // println!("Recieving data");
        match check_receiver.recv() {
            Err(e) => {
                return Err(anyhow!(
                    "Failed to recv from pulse list modules callback {}",
                    e
                ))
            }
            Ok(ListResult::Error) => {
                warn!("{}", anyhow!("Failed to read module list"));
                break;
            }
            Ok(ListResult::End) => {
                debug!("BREAKING");
                break;
            }
            Ok(ListResult::Item((b, i))) => {
                if b {
                    present = Some(i);
                    break;
                }
            }
        }
    }

    if let Some(r) = present {
        info!("Module #{} ({} {}) already loaded", r, module_name, args);
        return Ok(r);
    }
    mainloop.lock();
    let (load_sender, load_receiver): (
        crossbeam_channel::Sender<Result<u32>>,
        crossbeam_channel::Receiver<Result<u32>>,
    ) = crossbeam_channel::unbounded();

    let load_module_callback = move |module_index: u32| {
        load_sender
            .send(Ok(module_index))
            .expect("send channel error");
    };
    introspector.load_module(module_name, args, load_module_callback);

    mainloop.unlock();

    let result = match load_receiver.recv() {
        Err(err) => Err(anyhow!("Failed to recv from pulse module callback {}", err)),
        Ok(Err(err)) => Err(anyhow!("Failed to load pulse module {}", err)),
        Ok(Ok(module_index)) => Ok(module_index),
    };

    mainloop.stop();
    result
}

pub fn unload_module(loop_module_id: u32) -> Result<()> {
    let (mut mainloop, pulse_context) = connect_pulse()?;

    let (sender, receiver): (
        crossbeam_channel::Sender<bool>,
        crossbeam_channel::Receiver<bool>,
    ) = crossbeam_channel::unbounded();

    let callback = move |result| {
        sender.send(result).expect("channel send error");
    };

    mainloop.lock();

    let mut introspector = pulse_context.introspect();
    introspector.unload_module(loop_module_id, callback);

    mainloop.unlock();

    let result = match receiver.recv() {
        Err(err) => Err(anyhow!("Failed to unload pulse module {}", err)),
        Ok(false) => Err(anyhow!("Failed to unload pulse module {}")),
        Ok(true) => Ok(()),
    };

    mainloop.stop();

    result
}

fn connect_pulse() -> Result<(pulse::mainloop::threaded::Mainloop, pulse::context::Context)> {
    let mut mainloop = pulse::mainloop::threaded::Mainloop::new()
        .ok_or_else(|| anyhow!("Pulse Mainloop Creation failed"))?;

    mainloop
        .start()
        .map_err(|err| anyhow!("Pulse Mainloop Start failed {}", err))?;

    mainloop.lock();

    let mut pulse_context: pulse::context::Context =
        pulse::context::Context::new(&mainloop, "Soundboard")
            .ok_or_else(|| anyhow!("Pulse Connection Callback failed"))?;

    pulse_context
        .connect(None, pulse::context::flags::NOFLAGS, None)
        .map_err(|err| anyhow!("Pulse Mainloop Creation failed {}", err))?;

    mainloop.unlock();

    loop {
        match pulse_context.get_state() {
            State::Ready => {
                trace!("connection: ready");
                break;
            }
            State::Failed => {
                trace!("connection: failed");
                return Err(anyhow!("Failed to connect to Pulse Server: Failed state"));
            }
            State::Terminated => {
                trace!("connection: terminated");
                return Err(anyhow!(
                    "Failed to connect to Pulse Server: Terminated state"
                ));
            }
            State::Connecting => {
                trace!("connection: connecting");
            }
            _ => {
                trace!("connection: unexpected state");
            }
        };

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok((mainloop, pulse_context))
}
