use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use oxilangtag::LanguageTag;
use walkdir::WalkDir;

mod_use::mod_use!(args);

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    let args = args::Args::parse().inner_args();

    if let Err(e) = match args.command {
        Command::Update => {
            if Path::new(&args.root).is_dir() {
                update(&args.root)
            } else {
                Err("Invalid project root".into())
            }
        }
        Command::Generate { locales } => {
            if locales.is_empty() {
                Err("No locale specified to generate".into())
            } else {
                generate(locales, &args.root)
            }
        }
    } {
        eprintln!("Error: {}", e);
    }
}

fn extract_source(root: impl AsRef<Path>) -> Result<(HashSet<String>, String)> {
    let mut contents = HashSet::new();
    let mut locale_path = String::new();

    for entry in WalkDir::new(root.as_ref().join("src"))
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext == "rs")
                    .unwrap_or(false)
                    .then_some(entry)
            })
        })
    {
        r18_trans_support::source::extract(entry.path(), &mut contents, &mut locale_path)?;
    }

    Ok((contents, locale_path))
}

fn update(root: impl AsRef<Path>) -> Result<()> {
    let (contents, locale_path) = extract_source(root.as_ref())?;

    if locale_path.is_empty() {
        return Err("Missing translation directory".into());
    }

    if contents.is_empty() {
        println!("There is currently no untranslated text");
        return Ok(());
    }

    for entry in WalkDir::new(root.as_ref().join(locale_path))
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let mut parts = entry.path().file_name()?.to_str()?.split('.').rev();

                (parts.next() == Some("json")
                    && LanguageTag::parse(parts.next()?).is_ok()
                    && parts.next().is_none())
                    .then_some(entry)
            })
        })
    {
        let file_name = entry
            .path()
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        println!("\nChecking {file_name} for untranslated texts...");

        let mut is_modified = false;
        let mut todo = HashMap::new();
        let translation = r18_trans_support::translation::extract(entry.path());

        for content in contents.iter() {
            if !translation.contains_key(content) {
                is_modified = true;
                todo.insert(content.to_string(), "[TODO]".to_string());
            }
        }

        if is_modified {
            println!("{} untranslated text(s) were found", todo.len());
            println!("Writing to TODO.{}", file_name);

            todo.extend(translation.into_iter());
            r18_trans_support::translation::generate(entry.path(), todo)?;
        } else {
            println!("No untranslated text found");
        }
    }

    println!("\nDone");

    Ok(())
}

fn generate(locales: Vec<String>, root: impl AsRef<Path>) -> Result<()> {
    let (contents, locale_path) = extract_source(root.as_ref())?;

    if locale_path.is_empty() {
        return Err("Missing translation directory".into());
    }

    if contents.is_empty() {
        println!("There is currently no untranslated text");
        return Ok(());
    }

    for locale in locales {
        let locale = LanguageTag::parse_and_normalize(&locale)
            .map_err(|e| format!("Invalid locale: {}: {}", locale, e))?;

        let expected_name = format!("{}.json", locale);
        let expected_path = root.as_ref().join(&locale_path).join(&expected_name);

        if expected_path.exists() {
            println!(
                "{} already exists, please use update subcommand instead",
                expected_name
            );
            continue;
        }

        // TODO: automatic translation
        let todo = contents
            .iter()
            .map(|content| (content.clone(), "[TODO]".to_string()))
            .collect::<HashMap<_, _>>();

        r18_trans_support::translation::generate(expected_path, todo)?;
    }

    println!("\nDone");

    Ok(())
}
