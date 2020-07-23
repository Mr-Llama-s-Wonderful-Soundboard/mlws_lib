use std::{
    fmt::{Debug, Display},
    path::PathBuf,
    process::Command,
};

fn load_module(name: &str, args: &[&str]) -> String {
    let o = Command::new("pactl")
        .arg("load-module")
        .arg(name)
        .args(args)
        .output()
        .expect("failed to execute process");
    let stdout_string = String::from_utf8(o.stdout).expect("Unexpected non UTF8 output");
    String::from(stdout_string.trim())
}

fn unload_module(id: &str) {
    Command::new("pactl")
        .arg("unload-module")
        .arg(id)
        .output()
        .expect("failed to execute process");
    //let stdout_string = String::from_utf8(o.stdout).expect("Unexpected non UTF8 output");
}

pub struct Module {
    id: String,
    name: String,
    unloaded: bool,
}

impl Module {
    pub fn load(name: &str, args: &[&str]) -> Self {
        Self {
            id: load_module(name, args),
            name: String::from(name),
            unloaded: false,
        }
    }

    pub fn unload(&mut self) {
        if !self.unloaded {
            println!("Unloading {}", self);
            unload_module(&self.id);
            self.unloaded = true;
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        self.unload()
    }
}

impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Module {}[{}]", self.name, self.id)
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.name, self.id)
    }
}

pub fn play_file(device: &str, path: &PathBuf) {
    println!("{} {:?}", device, path.as_os_str());
    let mut cmd = Command::new("paplay");
    let cmd_arged = cmd.arg(format!("--device={}", device)).arg(path);
    //println!("{}",cmd_arged);
    //let out =
    cmd_arged.output().expect("failed to execute process");

    // let stdout_string = String::from_utf8(out.stdout).expect("Unexpected non UTF8 output");
    // println!("STDOUT: {}", stdout_string);
    // let stderr_string = String::from_utf8(out.stderr).expect("Unexpected non UTF8 output");
    // println!("STDERR: {}", stderr_string);
}
