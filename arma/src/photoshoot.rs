use arma_rs::{Context, ContextState, Group};
use hemtt_common::arma::control::fromarma::{Message, Photoshoot};

pub fn group() -> Group {
    Group::new()
        .command("ready", ready)
        .command("weapon", weapon)
        .command("previews", previews)
}

fn ready(ctx: Context) {
    let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
        println!("`photoshoot:ready` called without a sender");
        return;
    };
    sender.send(Message::Photoshoot(Photoshoot::Ready)).unwrap();
}

fn weapon(ctx: Context, weapon: String) {
    let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
        println!("`photoshoot:weapon` called without a sender");
        return;
    };
    sender
        .send(Message::Photoshoot(Photoshoot::Weapon(weapon)))
        .unwrap();
}

fn previews(ctx: Context) {
    let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
        println!("`photoshoot:previews` called without a sender");
        return;
    };
    sender
        .send(Message::Photoshoot(Photoshoot::Previews))
        .unwrap();
}
