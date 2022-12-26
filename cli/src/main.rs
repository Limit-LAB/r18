use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

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
        Command::Generate { locale } => todo!(),
    } {
        eprintln!("Error: {}", e);
    }
}

fn update(root: impl AsRef<Path>) -> Result<()> {
    let mut contents = HashSet::new();
    let mut locale = String::new();

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
        r18_trans_support::source::extract(entry.path(), &mut contents, &mut locale)?;
    }

    if locale.is_empty() {
        return Err("Missing translation directory".into());
    }

    if contents.is_empty() {
        println!("There is currently no untranslated text");
        return Ok(());
    }

    for entry in WalkDir::new(root.as_ref().join(locale))
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let path = entry.path();
                (path.extension() == Some("json".as_ref())
                    && !path
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .starts_with("TODO"))
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
