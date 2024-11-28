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
        Preview(String),
        PreviewRun,
        Done,
    }
}

pub mod fromarma {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Message {
        Control(Control),
        Photoshoot(Photoshoot),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Control {
        Mission(String),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Photoshoot {
        Ready,
        Weapon(String),
        Previews,
    }
}
