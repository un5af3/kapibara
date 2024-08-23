//! Handle serialize and deserialize

use serde::{Deserialize, Serialize};

use crate::OptionError;

#[derive(Debug)]
pub enum Codec {
    Json,
    Yaml,
}

impl Codec {
    pub fn from_str<'a, T>(&self, s: &'a str) -> Result<T, OptionError>
    where
        T: Deserialize<'a>,
    {
        let result = match self {
            Codec::Json => {
                serde_json::from_str(s).map_err(|e| OptionError::Deserialize(e.to_string()))?
            }
            Codec::Yaml => {
                serde_yaml::from_str(s).map_err(|e| OptionError::Deserialize(e.to_string()))?
            }
        };

        Ok(result)
    }

    pub fn to_string<T>(&self, value: &T) -> Result<String, OptionError>
    where
        T: ?Sized + Serialize,
    {
        let result =
            match self {
                Codec::Json => serde_json::to_string(value)
                    .map_err(|e| OptionError::Serialize(e.to_string()))?,
                Codec::Yaml => serde_yaml::to_string(value)
                    .map_err(|e| OptionError::Serialize(e.to_string()))?,
            };

        Ok(result)
    }
}
