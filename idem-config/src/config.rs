use std::fs::File;
use serde::de::DeserializeOwned;
use std::path::MAIN_SEPARATOR_STR;

#[derive(Default)]
pub struct Config<C> {
    config: C,
}

impl<C> Config<C>
where
    C: Default + DeserializeOwned
{
    pub fn new(provider: impl ConfigProvider<C>) -> Result<Self, ()> {
        match provider.load() {
            Ok(config) => Ok(Self{config}),
            Err(_) => todo!("Handle config provider error...")
        }
    }

    pub fn get(&self) -> &C {
        &self.config
    }

    pub fn get_mut(&mut self) -> &mut C {
        &mut self.config
    }
}

pub trait ConfigProvider<C>
where
    C: Default + DeserializeOwned
{
    fn load(&self) -> Result<C, ()>;
}

pub struct DefaultConfigProvider;

impl<C> ConfigProvider<C> for DefaultConfigProvider
where
    C: Default + DeserializeOwned
{
    fn load(&self) -> Result<C, ()> {
        Ok(C::default())
    }
}

pub struct FileConfigProvider {
    base_path: String,
    config_name: String
}

impl<C> ConfigProvider<C> for FileConfigProvider
where
    C: Default + DeserializeOwned
{
    fn load(&self) -> Result<C, ()> {
        let file = match File::open(format!("{}{}{}", self.base_path, MAIN_SEPARATOR_STR, self.config_name)) {
            Ok(file) => file,
            Err(_) => todo!("Handle failing to load configs... (file not found mostly)"),
        };
        match serde_json::from_reader(file) {
            Ok(config) => Ok(config),
            Err(_) => todo!("handle malformed config data from file...")
        }
    }
}