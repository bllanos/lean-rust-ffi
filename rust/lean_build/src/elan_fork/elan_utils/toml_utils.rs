use super::errors::*;
use toml;

pub fn get_string(table: &mut toml::value::Table, key: &str, path: &str) -> Result<String> {
    get_value(table, key, path).and_then(|v| {
        if let toml::Value::String(s) = v {
            Ok(s)
        } else {
            Err(Error::ExpectedType {
                expected_type: "string",
                path: path.to_owned(),
                key: key.to_owned(),
            })
        }
    })
}

pub fn get_opt_string(
    table: &mut toml::value::Table,
    key: &str,
    path: &str,
) -> Result<Option<String>> {
    if let Ok(v) = get_value(table, key, path) {
        if let toml::Value::String(s) = v {
            Ok(Some(s))
        } else {
            Err(Error::ExpectedType {
                expected_type: "string",
                path: path.to_owned(),
                key: key.to_owned(),
            })
        }
    } else {
        Ok(None)
    }
}

fn get_value(table: &mut toml::value::Table, key: &str, path: &str) -> Result<toml::Value> {
    table.remove(key).ok_or_else(|| Error::MissingKey {
        path: path.to_owned(),
        key: key.to_owned(),
    })
}

pub fn get_table(
    table: &mut toml::value::Table,
    key: &str,
    path: &str,
) -> Result<toml::value::Table> {
    if let Some(v) = table.remove(key) {
        if let toml::Value::Table(t) = v {
            Ok(t)
        } else {
            Err(Error::ExpectedType {
                expected_type: "table",
                path: path.to_owned(),
                key: key.to_owned(),
            })
        }
    } else {
        Ok(toml::value::Table::new())
    }
}
