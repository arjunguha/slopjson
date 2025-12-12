/// Formats a path component (key) for display in a JSON path.
/// Returns either `.key` format for valid identifiers or `["key"]` format for keys with spaces/special chars.
pub fn format_path_component(key: &str) -> String {
    // Check if key is a valid identifier (starts with letter/underscore, contains only alphanumeric/underscore)
    let is_valid_identifier = !key.is_empty() 
        && (key.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_'))
        && key.chars().all(|c| c.is_alphanumeric() || c == '_');
    
    if is_valid_identifier {
        format!(".{}", key)
    } else {
        // Escape quotes in the key for JSON string representation
        let escaped_key = key.replace('\\', "\\\\").replace('"', "\\\"");
        format!("[\"{}\"]", escaped_key)
    }
}

/// Builds a path by appending an object key to a base path.
pub fn build_object_path(base: &str, key: &str) -> String {
    let component = format_path_component(key);
    format!("{}{}", base, component)
}

/// Builds a path by appending an array index to a base path.
pub fn build_array_path(base: &str, index: usize) -> String {
    format!("{}[{}]", base, index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_path_component_simple_key() {
        assert_eq!(format_path_component("x"), ".x");
        assert_eq!(format_path_component("name"), ".name");
        assert_eq!(format_path_component("_private"), "._private");
        assert_eq!(format_path_component("key123"), ".key123");
    }

    #[test]
    fn test_format_path_component_key_with_spaces() {
        assert_eq!(format_path_component("my field"), "[\"my field\"]");
        assert_eq!(format_path_component("my key"), "[\"my key\"]");
        assert_eq!(format_path_component("  "), "[\"  \"]");
    }

    #[test]
    fn test_format_path_component_key_with_special_chars() {
        assert_eq!(format_path_component("key-name"), "[\"key-name\"]");
        assert_eq!(format_path_component("key.name"), "[\"key.name\"]");
        assert_eq!(format_path_component("key@value"), "[\"key@value\"]");
        assert_eq!(format_path_component("123key"), "[\"123key\"]");
    }

    #[test]
    fn test_format_path_component_key_with_quotes() {
        assert_eq!(format_path_component("key\"value"), "[\"key\\\"value\"]");
        assert_eq!(format_path_component("\"quoted\""), "[\"\\\"quoted\\\"\"]");
    }

    #[test]
    fn test_format_path_component_key_with_backslash() {
        assert_eq!(format_path_component("key\\value"), "[\"key\\\\value\"]");
    }

    #[test]
    fn test_format_path_component_empty_key() {
        assert_eq!(format_path_component(""), "[\"\"]");
    }

    #[test]
    fn test_build_object_path_simple() {
        assert_eq!(build_object_path("obj", "x"), "obj.x");
        assert_eq!(build_object_path("root", "name"), "root.name");
        assert_eq!(build_object_path("data", "value"), "data.value");
    }

    #[test]
    fn test_build_object_path_with_spaces() {
        assert_eq!(build_object_path("obj", "my field"), "obj[\"my field\"]");
        assert_eq!(build_object_path("root", "my key"), "root[\"my key\"]");
    }

    #[test]
    fn test_build_object_path_nested() {
        assert_eq!(build_object_path("obj.x", "y"), "obj.x.y");
        assert_eq!(build_object_path("obj[\"my field\"]", "nested"), "obj[\"my field\"].nested");
        assert_eq!(build_object_path("obj.x", "my field"), "obj.x[\"my field\"]");
    }

    #[test]
    fn test_build_array_path() {
        assert_eq!(build_array_path("arr", 0), "arr[0]");
        assert_eq!(build_array_path("arr", 42), "arr[42]");
        assert_eq!(build_array_path("obj.x", 0), "obj.x[0]");
    }

    #[test]
    fn test_build_array_path_nested() {
        assert_eq!(build_array_path("arr[0]", 1), "arr[0][1]");
        assert_eq!(build_array_path("obj[\"my field\"][0]", 1), "obj[\"my field\"][0][1]");
    }

    #[test]
    fn test_full_path_examples() {
        // Simple nested object
        let path1 = build_object_path("obj", "x");
        assert_eq!(path1, "obj.x");
        
        // Object with space in key
        let path2 = build_object_path("obj", "my field");
        assert_eq!(path2, "obj[\"my field\"]");
        
        // Nested: obj.x.y
        let path3 = build_object_path("obj.x", "y");
        assert_eq!(path3, "obj.x.y");
        
        // Nested: obj["my field"].nested
        let path4 = build_object_path("obj[\"my field\"]", "nested");
        assert_eq!(path4, "obj[\"my field\"].nested");
        
        // Array access: arr[0]
        let path5 = build_array_path("arr", 0);
        assert_eq!(path5, "arr[0]");
        
        // Mixed: obj.x[0]
        let path6 = build_array_path("obj.x", 0);
        assert_eq!(path6, "obj.x[0]");
        
        // Complex: obj["my field"][0].nested
        let path7 = build_array_path("obj[\"my field\"]", 0);
        let path8 = build_object_path(&path7, "nested");
        assert_eq!(path8, "obj[\"my field\"][0].nested");
        
        // Complex: obj.x["my field"]
        let path9 = build_object_path("obj.x", "my field");
        assert_eq!(path9, "obj.x[\"my field\"]");
    }

    #[test]
    fn test_path_generation_integration() {
        // Simulate building paths for a complex JSON structure
        // {
        //   "simple": "value1",
        //   "my field": "value2",
        //   "key-name": "value3",
        //   "123invalid": "value4",
        //   "normal_key": {
        //     "nested": "value5",
        //     "nested field": "value6"
        //   },
        //   "arr": [{"item": "value7"}]
        // }
        
        let root = "root";
        
        // Test all the object keys
        assert_eq!(build_object_path(root, "simple"), "root.simple");
        assert_eq!(build_object_path(root, "my field"), "root[\"my field\"]");
        assert_eq!(build_object_path(root, "key-name"), "root[\"key-name\"]");
        assert_eq!(build_object_path(root, "123invalid"), "root[\"123invalid\"]");
        assert_eq!(build_object_path(root, "normal_key"), "root.normal_key");
        
        // Test nested paths
        let nested_base = build_object_path(root, "normal_key");
        assert_eq!(build_object_path(&nested_base, "nested"), "root.normal_key.nested");
        assert_eq!(build_object_path(&nested_base, "nested field"), "root.normal_key[\"nested field\"]");
        
        // Test array paths
        let arr_base = build_object_path(root, "arr");
        assert_eq!(build_array_path(&arr_base, 0), "root.arr[0]");
        
        // Test nested array object
        let item_base = build_array_path(&arr_base, 0);
        assert_eq!(build_object_path(&item_base, "item"), "root.arr[0].item");
    }
}
