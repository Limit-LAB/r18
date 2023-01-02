use std::{collections::HashMap, fs::File, io::Read, path::Path};

use serde_json::Value;

pub fn extract(path: impl AsRef<Path>) -> crate::Result<HashMap<String, String>> {
    let mut content = String::new();

    File::open(path.as_ref())?.read_to_string(&mut content)?;

    let root = serde_json::from_str::<Value>(&content)?;

    Ok(extract_value(String::new(), root))
}

fn extract_value(prefix: String, object: Value) -> HashMap<String, String> {
    let mut ret = HashMap::new();

    macro_rules! gen_prefix {
        ($prefix:expr, $current:expr, $value:expr) => {
            match $value.is_object() || $value.is_array() {
                true => format!("{}.{}", $prefix, $current),
                false => format!("{} {}", $prefix, $current),
            }
        };
    }

    match object {
        Value::Null => {}
        Value::Bool(b) => {
            ret.insert(prefix, b.to_string());
        }
        Value::Number(n) => {
            ret.insert(prefix, n.to_string());
        }
        Value::String(s) => {
            ret.insert(prefix, s);
        }
        Value::Array(arr) => arr.into_iter().enumerate().for_each(|(i, v)| {
            ret.extend(extract_value(gen_prefix!(prefix, i, v), v));
        }),
        Value::Object(obj) => obj
            .into_iter()
            .for_each(|(k, v)| ret.extend(extract_value(gen_prefix!(prefix, k, v), v))),
    }

    ret
}

#[cfg(test)]
mod tests {
    #[test]
    fn extract_value_test() {
        let json = serde_json::json!({
            "Hello, {}": "你好，{}",
            "Debug: {:?}": "调试：{:?}",
            "{} is typing": "{} 正在输入",
            "evil": {
                "{} is typing": "{} 正在女装"
            }
        });

        assert_eq!(
            super::extract_value(String::new(), json),
            [
                (" Hello, {}", "你好，{}"),
                (" Debug: {:?}", "调试：{:?}"),
                (" {} is typing", "{} 正在输入"),
                (".evil {} is typing", "{} 正在女装")
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
        )
    }
}
