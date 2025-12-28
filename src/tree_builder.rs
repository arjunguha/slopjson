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

use crate::path_formatting::{build_array_path, build_object_path};
use crate::value_formatting::format_value_preview;
use glib::ToValue;
use gtk::prelude::{TreeStoreExt, TreeStoreExtManual};
use gtk::{TreeIter, TreeStore};
use serde_json::Value;

/// Sets all column values for a tree node.
///
/// # Arguments
///
/// * `tree_store` - The tree store to update
/// * `iter` - The tree iterator for the node
/// * `name` - Display name for the node (key or index)
/// * `value` - The JSON value to store
/// * `display_path` - The JSON path string for display in the UI
/// * `data_path` - The JSON path string for value lookup
/// * `doc_id` - Identifier for the source document
pub fn set_tree_node_values(
    tree_store: &TreeStore,
    iter: &TreeIter,
    name: &str,
    value: &Value,
    display_path: &str,
    data_path: &str,
    doc_id: i64,
) {
    tree_store.set_value(iter, 0, &name.to_value());
    tree_store.set_value(iter, 1, &format_value_preview(value).to_value());
    tree_store.set_value(iter, 2, &display_path.to_value());
    tree_store.set_value(iter, 3, &data_path.to_value());
    tree_store.set_value(iter, 4, &doc_id.to_value());
}

/// Recursively populates a tree store with JSON values.
///
/// # Arguments
///
/// * `tree_store` - The tree store to populate
/// * `parent` - The parent iterator (None for root)
/// * `value` - The JSON value to add
/// * `display_path` - The JSON path string for display in the UI
/// * `data_path` - The JSON path string for value lookup
/// * `doc_id` - Identifier for the source document
pub fn populate_tree(
    tree_store: &TreeStore,
    parent: &TreeIter,
    value: &Value,
    display_path: &str,
    data_path: &str,
    doc_id: i64,
) {
    match value {
        Value::Object(map) => {
            for (key, val) in map.iter() {
                let iter = tree_store.append(Some(parent));
                let new_display_path = build_object_path(display_path, key);
                let new_data_path = build_object_path(data_path, key);
                set_tree_node_values(
                    tree_store,
                    &iter,
                    key,
                    val,
                    &new_display_path,
                    &new_data_path,
                    doc_id,
                );
                populate_tree(
                    tree_store,
                    &iter,
                    val,
                    &new_display_path,
                    &new_data_path,
                    doc_id,
                );
            }
        }
        Value::Array(arr) => {
            for (idx, val) in arr.iter().enumerate() {
                let iter = tree_store.append(Some(parent));
                let new_display_path = build_array_path(display_path, idx);
                let new_data_path = build_array_path(data_path, idx);
                let name = format!("[{}]", idx);
                set_tree_node_values(
                    tree_store,
                    &iter,
                    &name,
                    val,
                    &new_display_path,
                    &new_data_path,
                    doc_id,
                );
                populate_tree(
                    tree_store,
                    &iter,
                    val,
                    &new_display_path,
                    &new_data_path,
                    doc_id,
                );
            }
        }
        _ => {
            // Leaf value, already set in parent call
        }
    }
}

/// Adds a single JSON value to the tree store as a root node.
///
/// # Arguments
///
/// * `tree_store` - The tree store to add to
/// * `value` - The JSON value to add
/// * `root_name` - Display name for the root node
pub fn add_single_value_to_tree(
    tree_store: &TreeStore,
    value: &Value,
    root_name: &str,
    doc_id: i64,
) {
    let root_iter = tree_store.append(None);
    // Use "$" as the root path for single objects/arrays (JSONPath notation)
    // This ensures proper path generation for nested structures
    let root_path = "$";
    set_tree_node_values(
        tree_store,
        &root_iter,
        root_name,
        value,
        root_path,
        root_path,
        doc_id,
    );
    populate_tree(tree_store, &root_iter, value, root_path, root_path, doc_id);
}

/// Adds a JSONL result to the tree store.
///
/// # Arguments
///
/// * `tree_store` - The tree store to add to
/// * `json_values` - The array of JSON values from the JSONL file
/// * `display_name` - Display name for the root node
/// * `display_root_path` - The root path string (typically the file name)
/// * `doc_id` - Identifier for the source document
pub fn add_jsonl_to_tree(
    tree_store: &TreeStore,
    json_values: &[Value],
    display_name: &str,
    display_root_path: &str,
    doc_id: i64,
) {
    let root_iter = tree_store.append(None);
    let root_name = format!("{} (JSONL)", display_name);
    tree_store.set_value(&root_iter, 0, &root_name.to_value());
    tree_store.set_value(
        &root_iter,
        1,
        &format!("{} objects", json_values.len()).to_value(),
    );
    tree_store.set_value(&root_iter, 2, &display_root_path.to_value());
    tree_store.set_value(&root_iter, 3, &"$".to_value());
    tree_store.set_value(&root_iter, 4, &doc_id.to_value());

    for (idx, value) in json_values.iter().enumerate() {
        let line_iter = tree_store.append(Some(&root_iter));
        let display_path = build_array_path(display_root_path, idx);
        let name = format!("Line {}", idx + 1);
        let data_path = build_array_path("$", idx);
        set_tree_node_values(
            tree_store,
            &line_iter,
            &name,
            value,
            &display_path,
            &data_path,
            doc_id,
        );
        populate_tree(
            tree_store,
            &line_iter,
            value,
            &display_path,
            &data_path,
            doc_id,
        );
    }
}

// Note: Tree building functions are tightly coupled to GTK and require GTK initialization.
// Integration tests for these functions would require GTK to be initialized, which is
// complex in a test environment. The core logic (path building, value formatting) is
// tested separately in their respective modules.

#[cfg(test)]
mod tests {
    use super::*;

    fn read_rss_kb() -> Option<usize> {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                let kb = rest.trim().split_whitespace().next()?.parse().ok()?;
                return Some(kb);
            }
        }
        None
    }

    #[test]
    fn test_large_string_tree_store_memory_usage() {
        if gtk::init().is_err() {
            return;
        }

        let large_value = Value::String("a".repeat(8 * 1024 * 1024));
        let tree_store = TreeStore::new(&[
            glib::Type::STRING,
            glib::Type::STRING,
            glib::Type::STRING,
            glib::Type::STRING,
            glib::Type::I64,
        ]);

        let before_kb = read_rss_kb().unwrap_or(0);
        add_single_value_to_tree(&tree_store, &large_value, "root", 0);
        let after_kb = read_rss_kb().unwrap_or(before_kb);
        let delta_kb = after_kb.saturating_sub(before_kb);

        assert!(
            delta_kb < 6 * 1024,
            "TreeStore growth too large: {} KB",
            delta_kb
        );
    }
}
