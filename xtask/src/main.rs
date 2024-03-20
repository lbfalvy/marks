use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

use clap::Parser;
use common::clone;

#[derive(clap::Parser, Debug)]
struct Args {
  #[command(subcommand)]
  pub cmd: Cmd,
}

#[derive(clap::Subcommand, Debug)]
enum Cmd {
  Run,
  Diesel {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    args: Vec<String>,
  },
}

fn main() -> ExitCode {
  // todo: set cwd to project root
  let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  root.pop();
  let client_dir = root.join("client");
  let server_dir = root.join("server");
  match Args::parse().cmd {
    Cmd::Run => {
      let server = Arc::new(Mutex::new(
        Command::new("cargo").args(["run"]).current_dir(server_dir).spawn().unwrap(),
      ));
      let client = Arc::new(Mutex::new(
        Command::new("trunk").args(["serve"]).current_dir(client_dir).spawn().unwrap(),
      ));
      thread::scope(|scope| {
        let (snd_e, recv_e) = channel();
        let snd = Arc::new(snd_e);
        {
          let (client, server) = (client.clone(), server.clone());
          scope.spawn(clone!(client, snd; move || {
            client.lock().unwrap().wait().unwrap();
            snd.send(()).unwrap()
          }));
          scope.spawn(clone!(server, snd; move || {
            server.lock().unwrap().wait().unwrap();
            snd.send(()).unwrap()
          }));
        }
        ctrlc::set_handler(move || snd.send(()).unwrap()).unwrap();
        recv_e.recv().unwrap();
        let _ = client.lock().unwrap().kill();
        let _ = server.lock().unwrap().kill();
      });
      ExitCode::SUCCESS
    },
    Cmd::Diesel { args } => ExitCode::from(
      Command::new("diesel")
        .args(args)
        .current_dir(server_dir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .code()
        .unwrap_or(1) as u8,
    ),
  }
}
