use arma_rs::{Context, ContextState, Group};
use hemtt_common::arma::control::fromarma::{Message, Photoshoot};

pub fn group() -> Group {
    Group::new()
        .group(
            "items",
            Group::new()
                .command("ready", items::ready)
                .command("weapon", items::weapon)
                .command("weapon_unsupported", items::weapon_unsupported)
                .command("vehicle", items::vehicle)
                .command("vehicle_unsupported", items::vehicle_unsupported),
        )
        .group(
            "previews",
            Group::new()
                .command("ready", previews::ready)
                .command("class", previews::class)
                .command("done", previews::done),
        )
}

mod items {
    use super::*;

    pub fn ready(ctx: Context) {
        let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
            println!("`photoshoot:ready` called without a sender");
            return;
        };
        sender
            .send(Message::Photoshoot(Photoshoot::ItemsReady))
            .unwrap();
    }

    pub fn weapon(ctx: Context, weapon: String) {
        let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
            println!("`photoshoot:items:weapon` called without a sender");
            return;
        };
        sender
            .send(Message::Photoshoot(Photoshoot::Weapon(weapon)))
            .unwrap();
    }

    pub fn weapon_unsupported(ctx: Context, weapon: String) {
        let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
            println!("`photoshoot:items:weapon_unsupported` called without a sender");
            return;
        };
        sender
            .send(Message::Photoshoot(Photoshoot::WeaponUnsupported(weapon)))
            .unwrap();
    }

    pub fn vehicle(ctx: Context, vehicle: String) {
        let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
            println!("`photoshoot:vehicle` called without a sender");
            return;
        };
        sender
            .send(Message::Photoshoot(Photoshoot::Vehicle(vehicle)))
            .unwrap();
    }

    pub fn vehicle_unsupported(ctx: Context, vehicle: String) {
        let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
            println!("`photoshoot:vehicle_unsupported` called without a sender");
            return;
        };
        sender
            .send(Message::Photoshoot(Photoshoot::VehicleUnsupported(vehicle)))
            .unwrap();
    }
}

mod previews {
    use super::*;

    pub fn ready(ctx: Context) {
        let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
            println!("`photoshoot:previews:ready` called without a sender");
            return;
        };
        sender
            .send(Message::Photoshoot(Photoshoot::PreviewsReady))
            .unwrap();
    }

    pub fn class(ctx: Context, class: String) {
        let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
            println!("`photoshoot:previews:class` called without a sender");
            return;
        };
        sender
            .send(Message::Photoshoot(Photoshoot::Previews(class)))
            .unwrap();
    }

    pub fn done(ctx: Context) {
        let Some(sender) = ctx.global().get::<std::sync::mpsc::Sender<Message>>() else {
            println!("`photoshoot:previews:done` called without a sender");
            return;
        };
        sender
            .send(Message::Photoshoot(Photoshoot::PreviewsDone))
            .unwrap();
    }
}
