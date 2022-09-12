use std::collections::HashMap;

use log::error;
use serde::{Deserialize, Serialize};
use yewdux::{prelude::*, storage};

use crate::{crud_instance::CrudInstanceConfig, CrudMainTrait};

#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct InstanceStore<T: CrudMainTrait> {
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    instances: HashMap<String, CrudInstanceConfig<T>>,
}

impl<T: CrudMainTrait> InstanceStore<T> {
    pub fn get(&self, instance_name: &str) -> Option<CrudInstanceConfig<T>> {
        self.instances.get(instance_name).cloned()
    }

    pub fn save(&mut self, instance_name: String, instance_config: CrudInstanceConfig<T>) {
        self.instances.insert(instance_name, instance_config);
    }
}

impl<T: 'static + CrudMainTrait> Store for InstanceStore<T> {
    fn new() -> Self {
        init_listener(storage::StorageListener::<Self>::new(storage::Area::Local));

        storage::load(storage::Area::Local)
            .map_err(|error| {
                // TODO: Erase from local store
                error!("Unable to load state due to StorageError: {}", error);
            })
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    fn should_notify(&self, other: &Self) -> bool {
        self != other
    }
}
