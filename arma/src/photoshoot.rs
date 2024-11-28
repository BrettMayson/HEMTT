use arma_rs::Group;
use hemtt_common::arma::control::fromarma::{Message, Photoshoot};

use crate::conn::Conn;

pub fn group() -> Group {
    Group::new()
        .command("ready", ready)
        .command("weapon", weapon)
        .command("previews", previews)
}

fn ready() {
    Conn::get()
        .send(Message::Photoshoot(Photoshoot::Ready))
        .unwrap();
}

fn weapon(weapon: String) {
    Conn::get()
        .send(Message::Photoshoot(Photoshoot::Weapon(weapon)))
        .unwrap();
}

fn previews() {
    Conn::get()
        .send(Message::Photoshoot(Photoshoot::Previews))
        .unwrap();
}
