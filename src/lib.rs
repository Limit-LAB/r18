use std::{collections::HashMap, sync::Mutex};

pub use once_cell::sync::Lazy;
pub use dynfmt::{Format, SimpleCurlyFormat};

pub use proc_macros::init;

pub struct Locale {
    pub name: &'static str,
    pub translate: HashMap<&'static str, &'static str>,
}

pub static CURRENT_LOCALE: Lazy<Mutex<Option<&'static Lazy<Locale>>>> =
    Lazy::new(|| Mutex::new(None));

pub fn translate<'a>(prefix: impl AsRef<str>, content: &'a str) -> &'a str {
    let locale = CURRENT_LOCALE.lock().unwrap();
    let locale = match *locale {
        Some(l) => l,
        None => return content,
    };

    match locale
        .translate
        .get(format!("{}.{}", prefix.as_ref(), content).as_str())
    {
        Some(tr) => tr,
        None => content,
    }
}

mod_use::mod_use!(macros);
