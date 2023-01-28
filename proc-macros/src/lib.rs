#![cfg_attr(feature = "nightly-features", feature(track_path))]
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use oxilangtag::LanguageTag;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use serde::Deserialize;
use walkdir::WalkDir;

struct PathStr(String);

impl syn::parse::Parse for PathStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::LitStr>().map(|v| Self(v.value()))
    }
}

#[derive(Debug, Default, Deserialize)]
struct Config {
    fallback: HashMap<String, String>,
}

struct LocaleExtra {
    name: String,
    ident: Ident,
    translations: HashMap<String, String>,
}

type LocaleModel = BTreeMap<
    String, // region
    LocaleExtra,
>;
type TranslationModel = HashMap<
    String, // primary language
    LocaleModel,
>;

/// Generate translation models and functions `set_locale` and `locale` to setup
/// `r18` environment with given translation directory.
///
/// ## Example
///
/// ```ignore
/// r18::init!("tr");
/// ```
#[proc_macro]
pub fn init(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let path = match syn::parse::<PathStr>(input) {
        Ok(dir) => {
            let p = std::env::var("CARGO_MANIFEST_DIR")
                .map(|p| PathBuf::from(p).join(dir.0))
                .expect("CARGO_MANIFEST_DIR doesn't exist");

            match p.is_dir() {
                true => p,
                false => panic!("{} is not a directory.", p.display()),
            }
        }
        Err(e) => return e.to_compile_error().into(),
    };

    let (config, model) = scan_locales(path);
    let locales = generate_locales(&model);
    let locale_helpers = generate_helpers(&config, &model);

    quote! {
        #[doc(hidden)]
        pub(crate) mod __r18_gen {
            #locales
            #locale_helpers
        }
    }
    .into()
}

fn scan_locales(path: impl AsRef<Path>) -> (Config, TranslationModel) {
    let mut model = TranslationModel::new();
    let mut config = Config::default();

    WalkDir::new(path)
        .into_iter()
        .filter_map(|p| {
            let path = p.ok()?;
            let mut parts = path.path().file_name()?.to_str()?.split('.').rev();

            let language = match (parts.next(), parts.next(), parts.next()) {
                (Some("json"), Some("config"), None) => {
                    config = load_config(path.path());
                    None
                }
                (Some("json"), Some(tag), None) => LanguageTag::parse_and_normalize(tag).ok(),
                _ => None,
            }?;

            Some((path, language))
        })
        .for_each(|(path, language)| {
            #[cfg(feature = "nightly-features")]
            proc_macro::tracked_path::path(path.path().to_str().unwrap_or_default());

            let region = language.region().unwrap_or_default().to_string();
            let name = region
                .is_empty()
                .then_some(String::new())
                .unwrap_or_else(|| format!("{}-{}", language.primary_language(), region));

            let extra = LocaleExtra {
                ident: format_ident!("{}", name.replace('-', "_").to_uppercase()),
                name,
                translations: r18_trans_support::translation::extract(path.path()).unwrap(),
            };

            if let Some(locales) = model.get_mut(language.primary_language()) {
                locales.insert(region, extra);
            } else {
                model.insert(language.primary_language().into(), {
                    let mut locales = BTreeMap::new();
                    locales.insert(region, extra);
                    locales
                });
            }
        });

    (config, model)
}

fn load_config(path: impl AsRef<Path>) -> Config {
    File::open(path)
        .ok()
        .and_then(|f| serde_json::from_reader(BufReader::new(f)).ok())
        .unwrap_or_default()
}

fn generate_primary(locales: &LocaleModel) -> proc_macro2::TokenStream {
    locales
        .iter()
        .map(|(_, extra)| {
            let code = &extra.ident;
            let name = &extra.name;
            let translation = extra.translations.iter().map(|(k, v)| quote!( #k => #v ));

            quote! {
                #[doc(hidden)]
                const #code: r18::Locale = r18::Locale {
                    name: #name,
                    translate: {
                        use r18::phf;
                        phf::phf_map! {
                            #( #translation ),*
                        }
                    }
                };
            }
        })
        .collect()
}

fn generate_locales(model: &TranslationModel) -> proc_macro2::TokenStream {
    model
        .iter()
        .map(|(_, locales)| generate_primary(locales))
        .collect()
}

fn generate_lang_matches(
    config: &Config,
    primary: &String,
    locales: &LocaleModel,
) -> proc_macro2::TokenStream {
    let exact_matches =
        locales
            .iter()
            .filter(|(region, _)| !region.is_empty())
            .map(|(region, extra)| {
                let ident = &extra.ident;
                quote! { (#primary, Some(#region)) => Some(&#ident) , }
            });

    if let Some((_, extra)) = locales.first_key_value() {
        let fallback_region = config
            .fallback
            .get(primary)
            .and_then(|fallback| {
                LanguageTag::parse_and_normalize(fallback)
                    .ok()?
                    .region()
                    .map(|r| r.to_string())
            })
            .unwrap_or_default();

        let fallback_match = locales
            .get(&fallback_region)
            .map(|extra| {
                let ident = &extra.ident;
                quote! { (#primary, _) => Some(&#ident) , }
            })
            .unwrap_or_else(|| {
                let ident = &extra.ident;
                quote! { (#primary, _) => Some(&#ident) , }
            });

        exact_matches.chain([fallback_match].into_iter()).collect()
    } else {
        quote!()
    }
}

fn generate_helpers(config: &Config, model: &TranslationModel) -> proc_macro2::TokenStream {
    let matches = model
        .iter()
        .map(|(primary, locales)| generate_lang_matches(config, primary, locales))
        .collect::<proc_macro2::TokenStream>();

    quote! {
        #[doc(hidden)]
        pub(crate) fn set_locale(locale: impl AsRef<str>) {
            *r18::CURRENT_LOCALE
                .lock()
                .unwrap() = match r18::LanguageTag::parse_and_normalize(locale.as_ref()) {
                    Ok(lang) => {
                        match (lang.primary_language(), lang.region()) {
                            #matches
                            _ => None,
                        }
                    }
                    Err(_) => None,
                };
        }
    }
}
