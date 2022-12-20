use std::{collections::HashMap, sync::Mutex};

pub use once_cell::sync::Lazy;

pub use proc_macros::init;

pub struct Locale {
    pub name: &'static str,
    pub translate: HashMap<&'static str, &'static str>,
}

pub static CURRENT_LOCALE: Lazy<Mutex<Option<&'static Lazy<Locale>>>> =
    Lazy::new(|| Mutex::new(None));

pub fn translate(prefix: &str, content: &str) {}

mod_use::mod_use!(macros);
