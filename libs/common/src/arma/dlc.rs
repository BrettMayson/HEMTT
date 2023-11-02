//! Information about Arma 3 DLCs

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Debug, Serialize)]
/// DLCs that require opt-in.
pub enum DLC {
    #[serde(rename = "contact")]
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
    #[serde(rename = "spe")]
    /// Creator DLC: Spearhead 1944
    /// https://store.steampowered.com/app/1175380/Arma_3_Creator_DLC_Spearhead_1944/
    Spearhead1944,
}

impl DLC {
    #[must_use]
    /// Return the -mod paramater
    pub const fn to_mod(&self) -> &str {
        match self {
            Self::Contact => "contact",
            Self::GlobalMobilization => "gm",
            Self::PrairieFire => "vn",
            Self::IronCurtain => "csla",
            Self::WesternSahara => "ws",
            Self::Spearhead1944 => "spe",
        }
    }

    #[must_use]
    /// Return the appid
    pub const fn to_appid(&self) -> &str {
        match self {
            Self::Contact => "1021790",
            Self::GlobalMobilization => "1042220",
            Self::PrairieFire => "1227700",
            Self::IronCurtain => "1294440",
            Self::WesternSahara => "1681170",
            Self::Spearhead1944 => "1175380",
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
            Self::Spearhead1944 => "Creator DLC: Spearhead 1944",
        }
        .to_string()
    }
}

impl TryFrom<String> for DLC {
    type Error = String;
    fn try_from(dlc: String) -> Result<Self, Self::Error> {
        Ok(
            match dlc.to_lowercase().trim_start_matches("creator dlc: ") {
                "1021790" | "contact" => Self::Contact,
                "1042220"
                | "gm"
                | "global mobilization"
                | "global mobilization - cold war germany" => Self::GlobalMobilization,
                "1227700" | "vn" | "sog" | "prairie fire" | "s.o.g. prairie fire" => {
                    Self::PrairieFire
                }
                "1294440" | "csla" | "iron curtain" | "csla iron curtain" => Self::IronCurtain,
                "1681170" | "ws" | "western sahara" => Self::WesternSahara,
                "1175380" | "spe" | "spearhead" | "spearhead 1944" => Self::Spearhead1944,
                _ => return Err(format!("Unknown DLC: {dlc}")),
            },
        )
    }
}

impl<'de> Deserialize<'de> for DLC {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from(s).map_err(serde::de::Error::custom)
    }
}
