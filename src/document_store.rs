// Copyright (C) 2025 Arjun Guha
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::value_lookup::{lookup_value, lookup_value_in_jsonl};
use serde_json::Value;

#[derive(Debug)]
pub struct JsonLDocument {
    values: Vec<Value>,
    summary: Value,
}

impl JsonLDocument {
    pub fn new(values: Vec<Value>) -> Self {
        let summary = serde_json::json!({ "lines": values.len() });
        Self { values, summary }
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }
}

#[derive(Debug)]
pub enum StoredDocument {
    Single(Value),
    JsonL(JsonLDocument),
}

impl StoredDocument {
    pub fn lookup_value(&self, path: &str) -> Option<&Value> {
        match self {
            StoredDocument::Single(value) => lookup_value(value, path),
            StoredDocument::JsonL(doc) => {
                if path == "$" {
                    return Some(&doc.summary);
                }
                lookup_value_in_jsonl(&doc.values, path)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonl_summary_root() {
        let doc = StoredDocument::JsonL(JsonLDocument::new(vec![
            serde_json::json!({"name": "first"}),
            serde_json::json!({"name": "second"}),
        ]));
        let result = doc.lookup_value("$").unwrap();
        assert_eq!(result["lines"], 2);
    }
}
