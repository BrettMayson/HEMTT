use std::io::{Read, Write};

use arma_rs::{arma, Extension};
use conn::Conn;
use hemtt_arma::messages::{
    fromarma::{self, Control, Message},
    toarma,
};

mod conn;

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
            interprocess::local_socket::LocalSocketStream::connect("hemtt_arma").unwrap();
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
                        toarma::Photoshoot::Uniform(uniform) => {
                            println!("Uniform: {}", uniform);
                            ctx.callback_data("hemtt_photoshoot", "uniform", uniform.clone());
                        }
                        toarma::Photoshoot::Done => {
                            println!("Done");
                            ctx.callback_null("hemtt_photoshoot", "done");
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

mod photoshoot {
    use arma_rs::Group;
    use hemtt_arma::messages::fromarma::{Message, Photoshoot};

    use crate::conn::Conn;

    pub fn group() -> Group {
        Group::new()
            .command("ready", ready)
            .command("uniform", uniform)
    }

    fn ready() {
        Conn::get()
            .send(Message::Photoshoot(Photoshoot::Ready))
            .unwrap();
    }

    fn uniform(uniform: String) {
        Conn::get()
            .send(Message::Photoshoot(Photoshoot::Uniform(uniform)))
            .unwrap();
    }
}

fn mission(mission: String) {
    Conn::get()
        .send(Message::Control(Control::Mission(mission)))
        .unwrap();
}

fn send(message: fromarma::Message, socket: &mut interprocess::local_socket::LocalSocketStream) {
    let message = serde_json::to_string(&message).unwrap();
    socket
        .write_all(&u32::to_le_bytes(message.len() as u32))
        .unwrap();
    socket.write_all(message.as_bytes()).unwrap();
}
