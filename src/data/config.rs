use anyhow::{bail, Result};
use std::{collections::HashMap, io::Read};

// TODO Test this stuff or whatever
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum Theme {
    Nameless,
    Ika,
}
impl Theme {
    // returns an option incase we want more themes
    pub fn egui_theme(&self) -> Option<eframe::Theme> {
        Some(match self {
            Self::Ika => eframe::Theme::Light,
            Self::Nameless => eframe::Theme::Dark,
        })
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ConfigFs {
    theme: Option<Theme>,
    aliases: Option<HashMap<String, String>>,
    implied: Option<HashMap<String, String>>,
}

pub struct Config {
    pub theme: Theme,
    pub aliases: HashMap<String, String>,
    pub implied: HashMap<String, String>,
}
impl Config {
    pub fn from_fs(fs: ConfigFs) -> Self {
        Self {
            theme: fs.theme.unwrap_or(Theme::Ika),
            aliases: fs.aliases.unwrap_or_default(),
            implied: fs.implied.unwrap_or_default(),
        }
    }
    pub fn as_fs(&self) -> ConfigFs {
        ConfigFs {
            theme: Some(self.theme),
            aliases: Some(self.aliases.clone()),
            implied: Some(self.implied.clone()),
        }
    }
    pub fn load() -> Self {
        // None because there will only be one config
        let inner: fn() -> Result<ConfigFs> = || {
            let mut path = crate::database::tag_database::get_database_path(None)?;
            path.push("tag-maid.cfg");
            if path.exists() {
                let mut file = std::fs::File::open(path)?;
                let mut str = String::new();
                file.read_to_string(&mut str)?;
                let de: ConfigFs = serde_json::from_str(&str)?;
                return Ok(de);
            } else {
                bail!("cfg does not exist on disk");
            }
        };
        let res = inner();
        Self::from_fs(res.unwrap_or_default())
    }
    pub fn save(&self) -> Result<()> {
        let mut path = crate::database::tag_database::get_database_path(None)?;
        path.push("tag-maid.cfg");
        std::fs::write(path, serde_json::to_string(&self.as_fs())?)?;
        Ok(())
    }
}
