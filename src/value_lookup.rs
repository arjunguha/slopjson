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

use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

pub fn parse_json_path(path: &str) -> Option<Vec<PathSegment>> {
    let mut chars = path.chars().peekable();
    if chars.next()? != '$' {
        return None;
    }

    let mut segments = Vec::new();
    while let Some(&ch) = chars.peek() {
        match ch {
            '.' => {
                chars.next();
                let mut key = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '.' || c == '[' {
                        break;
                    }
                    key.push(c);
                    chars.next();
                }
                if key.is_empty() {
                    return None;
                }
                segments.push(PathSegment::Key(key));
            }
            '[' => {
                chars.next();
                if chars.peek() == Some(&'"') {
                    chars.next();
                    let mut key = String::new();
                    while let Some(c) = chars.next() {
                        match c {
                            '\\' => {
                                if let Some(escaped) = chars.next() {
                                    key.push(escaped);
                                } else {
                                    return None;
                                }
                            }
                            '"' => break,
                            _ => key.push(c),
                        }
                    }
                    if chars.next()? != ']' {
                        return None;
                    }
                    segments.push(PathSegment::Key(key));
                } else {
                    let mut index_str = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == ']' {
                            break;
                        }
                        if !c.is_ascii_digit() {
                            return None;
                        }
                        index_str.push(c);
                        chars.next();
                    }
                    if chars.next()? != ']' {
                        return None;
                    }
                    let index = index_str.parse::<usize>().ok()?;
                    segments.push(PathSegment::Index(index));
                }
            }
            _ => return None,
        }
    }

    Some(segments)
}

fn lookup_in_value<'a>(value: &'a Value, segments: &[PathSegment]) -> Option<&'a Value> {
    let mut current = value;
    for segment in segments {
        match segment {
            PathSegment::Key(key) => {
                current = current.get(key)?;
            }
            PathSegment::Index(index) => {
                current = current.get(*index)?;
            }
        }
    }
    Some(current)
}

pub fn lookup_value<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let segments = parse_json_path(path)?;
    if segments.is_empty() {
        return Some(root);
    }
    lookup_in_value(root, &segments)
}

pub fn lookup_value_in_jsonl<'a>(values: &'a [Value], path: &str) -> Option<&'a Value> {
    let segments = parse_json_path(path)?;
    if segments.is_empty() {
        return None;
    }
    let (first, rest) = segments.split_first()?;
    match first {
        PathSegment::Index(index) => {
            let value = values.get(*index)?;
            lookup_in_value(value, rest)
        }
        PathSegment::Key(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_path_simple() {
        let segments = parse_json_path("$.foo[0].bar").unwrap();
        assert_eq!(
            segments,
            vec![
                PathSegment::Key("foo".to_string()),
                PathSegment::Index(0),
                PathSegment::Key("bar".to_string())
            ]
        );
    }

    #[test]
    fn test_parse_json_path_with_bracket_key() {
        let segments = parse_json_path("$[\"my key\"][1]").unwrap();
        assert_eq!(
            segments,
            vec![
                PathSegment::Key("my key".to_string()),
                PathSegment::Index(1),
            ]
        );
    }

    #[test]
    fn test_parse_json_path_with_escaped_chars() {
        let segments = parse_json_path("$[\"key\\\"value\\\\test\"]").unwrap();
        assert_eq!(
            segments,
            vec![PathSegment::Key("key\"value\\test".to_string())]
        );
    }

    #[test]
    fn test_lookup_value_simple() {
        let value = serde_json::json!({"foo": [ {"bar": 1} ]});
        let result = lookup_value(&value, "$.foo[0].bar").unwrap();
        assert_eq!(result, &serde_json::json!(1));
    }

    #[test]
    fn test_lookup_value_root() {
        let value = serde_json::json!({"foo": "bar"});
        let result = lookup_value(&value, "$").unwrap();
        assert_eq!(result, &value);
    }

    #[test]
    fn test_lookup_value_jsonl() {
        let values = vec![
            serde_json::json!({"name": "first"}),
            serde_json::json!({"name": "second", "value": 42}),
        ];
        let result = lookup_value_in_jsonl(&values, "$[1].value").unwrap();
        assert_eq!(result, &serde_json::json!(42));
    }
}
