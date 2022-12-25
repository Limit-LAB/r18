use std::{collections::HashSet, path::Path};

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

    for entry in WalkDir::new(root.as_ref().join(locale))
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
        let mut is_modified = false;
        let mut translation = r18_trans_support::translation::extract(entry.path());

        for content in contents.iter() {
            if !translation.contains_key(content) {
                is_modified = true;
                translation.insert(content.into(), "[TODO]".into());
            }
        }

        if is_modified {

        }
    }

    Ok(())
}
