use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
/// DLCs that require opt-in.
pub enum DLC {
    #[serde(rename = "enoch")]
    /// Contact
    /// https://store.steampowered.com/app/1021790/Arma_3_Contact/
    Contact,
    #[serde(rename = "gm")]
    /// Creator DLC: Global Mobilization - Cold War Germany
    /// https://store.steampowered.com/app/1042220/Arma_3_Creator_DLC_Global_Mobilization__Cold_War_Germany/
    GlobalMobilization,
    #[serde(rename = "vn")]
    /// Creator DLC: S.O.G. Prairie Fire
    /// https://store.steampowered.com/app/1227700/Arma_3_Creator_DLC_SOG_Prairie_Fire/
    PrairieFire,
    #[serde(rename = "csla")]
    /// Creator DLC: CSLA Iron Curtain
    /// https://store.steampowered.com/app/1294440/Arma_3_Creator_DLC_CSLA_Iron_Curtain/
    IronCurtain,
    #[serde(rename = "ws")]
    /// Creator DLC: Western Sahara
    /// https://store.steampowered.com/app/1681170/Arma_3_Creator_DLC_Western_Sahara/
    WesternSahara,
}

impl DLC {
    #[must_use]
    /// Return the -mod paramater
    pub const fn to_mod(&self) -> &str {
        match self {
            Self::Contact => "enoch",
            Self::GlobalMobilization => "gm",
            Self::PrairieFire => "vn",
            Self::IronCurtain => "csla",
            Self::WesternSahara => "ws",
        }
    }
}

impl ToString for DLC {
    fn to_string(&self) -> String {
        match self {
            Self::Contact => "Contact",
            Self::GlobalMobilization => "Creator DLC: Global Mobilization - Cold War Germany",
            Self::PrairieFire => "Creator DLC: S.O.G. Prairie Fire",
            Self::IronCurtain => "Creator DLC: CSLA Iron Curtain",
            Self::WesternSahara => "Creator DLC: Western Sahara",
        }
        .to_string()
    }
}

impl TryFrom<String> for DLC {
    type Error = String;
    fn try_from(dlc: String) -> Result<Self, Self::Error> {
        Ok(
            match dlc.to_lowercase().trim_start_matches("creator dlc: ") {
                "contact" => Self::Contact,
                "gm" | "global mobilization" | "global mobilization - cold war germany" => {
                    Self::GlobalMobilization
                }
                "sog" | "prairie fire" | "s.o.g. prairie fire" => Self::PrairieFire,
                "csla" | "iron curtain" | "csla iron curtain" => Self::IronCurtain,
                "ws" | "western sahara" => Self::WesternSahara,
                _ => return Err(format!("Unknown DLC: {}", dlc)),
            },
        )
    }
}

impl<'de> Deserialize<'de> for DLC {
    fn deserialize<D>(deserializer: D) -> Result<DLC, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DLC::try_from(s).map_err(serde::de::Error::custom)
    }
}
