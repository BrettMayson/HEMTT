use std::io::{Read, Write};

use arma_rs::{arma, Context, ContextState, Extension};
use hemtt_common::arma::control::{
    fromarma::{self, Control, Message},
    toarma,
};
use interprocess::local_socket::{prelude::*, GenericNamespaced, Stream};

mod photoshoot;

#[arma]
fn init() -> Extension {
    let ext = Extension::build()
        .command("mission", mission)
        .command("log", log)
        .group("photoshoot", photoshoot::group())
        .finish();
    let ctx = ext.context();
    let (send, recv) = std::sync::mpsc::channel::<Message>();
    ctx.global().set(send);
    std::thread::spawn(move || {
        let mut socket =
            Stream::connect("hemtt_arma".to_ns_name::<GenericNamespaced>().unwrap()).unwrap();
        socket.set_nonblocking(true).unwrap();
        loop {
            let mut len_buf = [0u8; 4];
            if socket.read_exact(&mut len_buf).is_ok()
                && !len_buf.is_empty()
                && len_buf != [255u8; 4]
            {
                let len = u32::from_le_bytes(len_buf);
                println!("Receiving: {len}");
                let mut buf = vec![0u8; len as usize];
                socket.read_exact(&mut buf).unwrap();
                let buf = String::from_utf8(buf).unwrap();
                let message: toarma::Message = serde_json::from_str(&buf).unwrap();
                match message {
                    toarma::Message::Control(control) => match control {
                        toarma::Control::Exit => {
                            std::process::exit(0);
                        }
                    },
                    toarma::Message::Photoshoot(photoshoot) => match photoshoot {
                        toarma::Photoshoot::Weapon(weapon) => {
                            println!("Weapon: {weapon}");
                            ctx.callback_data("hemtt_photoshoot", "weapon_add", weapon.clone())
                                .unwrap();
                        }
                        toarma::Photoshoot::Preview(class) => {
                            println!("Preview: {class}");
                            ctx.callback_data("hemtt_photoshoot", "preview_add", class.clone())
                                .unwrap();
                        }
                        toarma::Photoshoot::PreviewRun => {
                            println!("PreviewRun");
                            ctx.callback_null("hemtt_photoshoot", "preview_run")
                                .unwrap();
                        }
                        toarma::Photoshoot::Done => {
                            println!("Done");
                            ctx.callback_null("hemtt_photoshoot", "done").unwrap();
                        }
                    },
                }
            }
            if let Ok(message) = recv.recv_timeout(std::time::Duration::from_millis(100)) {
                crate::send(message, &mut socket);
            }
        }
    });
    ext
}

fn mission(ctx: Context, mission: String) {
    let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
        println!("`mission` called without a sender");
        return;
    };
    sender
        .send(Message::Control(Control::Mission(mission)))
        .unwrap();
}

fn log(ctx: Context, level: String, message: String) {
    let level = match level.as_str() {
        "trace" => fromarma::Level::Trace,
        "debug" => fromarma::Level::Debug,
        "info" => fromarma::Level::Info,
        "warn" => fromarma::Level::Warn,
        "error" => fromarma::Level::Error,
        _ => {
            println!("Unknown log level: {}", level);
            fromarma::Level::Info
        }
    };
    let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
        println!("`log` called without a sender");
        return;
    };
    sender.send(Message::Log(level, message)).unwrap();
}

fn send(message: fromarma::Message, socket: &mut Stream) {
    let message = serde_json::to_string(&message).unwrap();
    let len = u32::try_from(message.len()).unwrap();
    socket.write_all(&u32::to_le_bytes(len)).unwrap();
    socket.write_all(message.as_bytes()).unwrap();
    socket.flush().unwrap();
}
