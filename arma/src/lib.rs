use std::io::{Read, Write};

use arma_rs::{arma, Extension};
use conn::Conn;
use hemtt_common::arma::control::{
    fromarma::{self, Control, Message},
    toarma,
};
use interprocess::local_socket::{prelude::*, GenericNamespaced, Stream};

mod conn;
mod photoshoot;

#[arma]
fn init() -> Extension {
    let ext = Extension::build()
        .command("mission", mission)
        .group("photoshoot", photoshoot::group())
        .finish();
    let ctx = ext.context();
    let (send, recv) = std::sync::mpsc::channel::<Message>();
    std::thread::spawn(move || {
        Conn::set(send);
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
                println!("Receiving: {}", len);
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
                            println!("Weapon: {}", weapon);
                            ctx.callback_data("hemtt_photoshoot", "weapon", weapon.clone())
                                .unwrap();
                        }
                        toarma::Photoshoot::Preview(class) => {
                            println!("Preview: {}", class);
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

fn mission(mission: String) {
    Conn::get()
        .send(Message::Control(Control::Mission(mission)))
        .unwrap();
}

fn send(message: fromarma::Message, socket: &mut Stream) {
    let message = serde_json::to_string(&message).unwrap();
    socket
        .write_all(&u32::to_le_bytes(message.len() as u32))
        .unwrap();
    socket.write_all(message.as_bytes()).unwrap();
    socket.flush().unwrap();
}
