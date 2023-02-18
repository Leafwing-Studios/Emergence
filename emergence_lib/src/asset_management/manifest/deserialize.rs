use bevy::{reflect::TypeUuid, utils::HashMap};
use serde::Deserialize;

use super::Manifest;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

impl<'de, Id, Data> Deserialize<'de> for Manifest<Id, Data>
where
    Id: Debug
        + Display
        + PartialEq
        + Eq
        + Hash
        + Send
        + Sync
        + TypeUuid
        + for<'d> Deserialize<'d>
        + 'static,
    Data: Debug + Send + Sync + TypeUuid + 'static + for<'d> Deserialize<'d>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = HashMap::deserialize(deserializer)?;
        Ok(Manifest::new(map))
    }
}
