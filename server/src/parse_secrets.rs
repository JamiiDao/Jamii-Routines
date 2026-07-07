use std::{fs::File, io::Read, sync::LazyLock};

use serde::{Deserialize, Serialize};

pub(crate) static SECRETS: LazyLock<Secrets> = LazyLock::new(|| Secrets::parse().unwrap());

#[derive(Serialize, Deserialize, Default, PartialEq)]
pub struct Secrets {
    smtps: String,
}

impl Secrets {
    pub fn parse() -> std::io::Result<Self> {
        #[cfg(debug_assertions)]
        let mut file = File::open("../secrets.toml")?;
        #[cfg(not(debug_assertions))]
        let mut file = File::open("./secrets.toml")?;
        let mut contents = String::default();
        file.read_to_string(&mut contents)?;

        toml::from_str(&contents).or(Err(std::io::ErrorKind::InvalidData.into()))
    }

    pub fn smtps(&self) -> &str {
        self.smtps.as_str()
    }

    // Ensures loaded during
    pub fn asserts() {
        if !(*SECRETS != Self::default()) {
            panic!("SECRETS NOT INITIALIZED PROPERLY")
        }
    }
}
