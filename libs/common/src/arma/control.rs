//! Messages to control Arma from HEMTT

pub mod toarma {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Message {
        Control(Control),
        Photoshoot(Photoshoot),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Control {
        Exit,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Photoshoot {
        Weapon(String),
        Vehicle(String),
        Preview(String),
        PreviewRun,
        Done,
    }
}

pub mod fromarma {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Level {
        Trace,
        Debug,
        Info,
        Warn,
        Error,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Message {
        Control(Control),
        Photoshoot(Photoshoot),
        Log(Level, String),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Control {
        Mission(String),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Photoshoot {
        ItemsReady,
        Weapon(String),
        WeaponUnsupported(String),
        Vehicle(String),
        VehicleUnsupported(String),

        PreviewsReady,
        PreviewsDone,
    }
}
