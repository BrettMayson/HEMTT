use arma_rs::{Context, ContextState, Group};
use hemtt_common::arma::control::fromarma::{Message, Photoshoot};

pub fn group() -> Group {
    Group::new()
        .command("ready", ready)
        .command("weapon", weapon)
        .command("weapon_unsupported", weapon_unsupported)
        .command("vehicle", vehicle)
        .command("vehicle_unsupported", vehicle_unsupported)
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

fn weapon_unsupported(ctx: Context, weapon: String) {
    let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
        println!("`photoshoot:weapon_unsupported` called without a sender");
        return;
    };
    sender
        .send(Message::Photoshoot(Photoshoot::WeaponUnsupported(weapon)))
        .unwrap();
}

fn vehicle(ctx: Context, vehicle: String) {
    let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
        println!("`photoshoot:vehicle` called without a sender");
        return;
    };
    sender
        .send(Message::Photoshoot(Photoshoot::Vehicle(vehicle)))
        .unwrap();
}

fn vehicle_unsupported(ctx: Context, vehicle: String) {
    let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
        println!("`photoshoot:vehicle_unsupported` called without a sender");
        return;
    };
    sender
        .send(Message::Photoshoot(Photoshoot::VehicleUnsupported(vehicle)))
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
