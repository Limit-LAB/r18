use std::{collections::HashMap, fs::File, io::Write, path::Path};

use serde_json::{Map, Value};

pub fn generate(
    old_path: impl AsRef<Path>,
    translation: HashMap<String, String>,
) -> crate::Result<()> {
    let document = generate_inner(translation)?;

    let todo_path = old_path.as_ref().with_file_name(format!(
        "TODO.{}",
        old_path
            .as_ref()
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
    ));

    let mut todo_file =
        File::create(todo_path).map_err(|e| format!("Failed to create todo file: {}", e))?;

    writeln!(
        todo_file,
        "{}",
        serde_json::to_string_pretty(&document).unwrap()
    )
    .map_err(|e| format!("Failed to write todo file: {}", e))?;

    Ok(())
}

fn generate_inner(translation: HashMap<String, String>) -> crate::Result<Value> {
    let mut document = Map::new().into();

    for (key, value) in translation {
        let (prefix, content) = key
            .split_once(' ')
            .ok_or("Can not find whitespace in key")?;
        let mut level = prefix
            .split('.')
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>();

        level.push(content);
        generate_value(level.into_iter(), &mut document, value);
    }

    Ok(document)
}

fn generate_value<'a>(mut level: impl Iterator<Item = &'a str>, parent: &mut Value, value: String) {
    match level.next() {
        Some(current) => generate_value(level, &mut parent[current], value),
        None => *parent = Value::String(value),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_generate() {
        let json = serde_json::json!({
            "Hello, {}": "你好，{}",
            "Debug: {:?}": "调试：{:?}",
            "{} is typing": "{} 正在输入",
            "First sentence. Second sentence.": "第一句话。第二句话。",
            "evil": {
                "{} is typing": "{} 正在女装"
            }
        });

        let translation = [
            (" Hello, {}", "你好，{}"),
            (" Debug: {:?}", "调试：{:?}"),
            (" {} is typing", "{} 正在输入"),
            (" First sentence. Second sentence.", "第一句话。第二句话。"),
            (".evil {} is typing", "{} 正在女装"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<String, String>>();

        assert_eq!(json, super::generate_inner(translation).unwrap());
    }
}
