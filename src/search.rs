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

/// Finds all occurrences of a pattern in text, returning (start, end) **character** offsets.
///
/// Important: GTK's `TextBuffer::iter_at_offset` expects offsets in *characters*, not bytes.
pub fn find_all_occurrences(
    text: &str,
    pattern: &str,
    case_sensitive: bool,
) -> Vec<(usize, usize)> {
    let mut occurrences = Vec::new();

    if pattern.is_empty() {
        return occurrences;
    }

    let pattern_len_chars = pattern.chars().count();
    if pattern_len_chars == 0 {
        return occurrences;
    }

    if case_sensitive {
        // Case-sensitive search:
        // `find` returns byte offsets, so we convert to character offsets.
        let mut start_byte = 0;
        while let Some(pos) = text[start_byte..].find(pattern) {
            let absolute_byte = start_byte + pos;
            let char_start = text[..absolute_byte].chars().count();
            let char_end = char_start + pattern_len_chars;
            occurrences.push((char_start, char_end));

            // Advance by one byte to allow overlapping-ish stepping while staying safe.
            // (This mirrors prior behavior and keeps UX intuitive.)
            start_byte = absolute_byte + 1;
            if start_byte >= text.len() {
                break;
            }
        }
    } else {
        // Case-insensitive search: iterate through characters and check each position.
        let pattern_lower: Vec<char> = pattern.to_lowercase().chars().collect();
        if pattern_lower.is_empty() {
            return occurrences;
        }

        let text_chars: Vec<(usize, char)> = text.char_indices().collect();

        for i in 0..text_chars.len() {
            if i + pattern_lower.len() > text_chars.len() {
                break;
            }

            // Check if substring starting at i matches pattern (case-insensitive)
            let mut matches = true;
            for (j, &pattern_char) in pattern_lower.iter().enumerate() {
                if i + j >= text_chars.len() {
                    matches = false;
                    break;
                }
                let text_char_lower = text_chars[i + j].1.to_lowercase().next().unwrap_or('\0');
                if text_char_lower != pattern_char {
                    matches = false;
                    break;
                }
            }

            if matches {
                // `i` is already a character index; return character offsets.
                let char_start = i;
                let char_end = i + pattern_lower.len();
                occurrences.push((char_start, char_end));
            }
        }
    }

    occurrences
}

/// Determines which occurrence to highlight in a formatted value
/// Returns the occurrence index (0-based) within the formatted_value that corresponds
/// to the match at match_index in the matches list for the same path
///
/// matches: List of (global_match_index, is_key_match) for matches with the same path
/// match_index: The global match index we want to highlight
pub fn find_occurrence_to_highlight(
    matches: &[(usize, bool)], // (global_match_index, is_key_match) for matches with same path
    match_index: usize,        // Global match index
    formatted_value: &str,
    search_text: &str,
    case_sensitive: bool,
) -> Option<(usize, usize)> {
    // Find the local index within path_matches that corresponds to match_index
    let local_index = matches.iter().position(|(idx, _)| *idx == match_index)?;

    // Bounds check to prevent panic
    if local_index >= matches.len() {
        return None;
    }

    // If this is a key match, don't highlight in the value
    if matches[local_index].1 {
        return None;
    }

    // Count how many value matches (not key matches) occur before this one in path_matches
    let mut occurrence_in_node = 0;
    for i in 0..local_index {
        if !matches[i].1 {
            // This is a value match, not a key match
            occurrence_in_node += 1;
        }
    }

    // Find all occurrences in the formatted value
    let occurrences = find_all_occurrences(formatted_value, search_text, case_sensitive);

    // Return the occurrence at the calculated index, with bounds check
    if occurrence_in_node < occurrences.len() {
        Some(occurrences[occurrence_in_node])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_all_occurrences_case_sensitive() {
        let text = "hello world hello";
        let pattern = "hello";
        let occurrences = find_all_occurrences(text, pattern, true);
        assert_eq!(occurrences, vec![(0, 5), (12, 17)]);
    }

    #[test]
    fn test_find_all_occurrences_case_insensitive() {
        let text = "Hello world HELLO";
        let pattern = "hello";
        let occurrences = find_all_occurrences(text, pattern, false);
        // Should find occurrences (at least the first "Hello")
        assert!(occurrences.len() >= 1);
        // Check first occurrence
        assert_eq!(occurrences[0], (0, 5)); // "Hello"
                                            // Verify the substring matches
        assert_eq!(&text[occurrences[0].0..occurrences[0].1], "Hello");
    }

    #[test]
    fn test_find_all_occurrences_empty_pattern() {
        let text = "hello world";
        let pattern = "";
        let occurrences = find_all_occurrences(text, pattern, true);
        assert_eq!(occurrences, vec![]);
    }

    #[test]
    fn test_find_all_occurrences_no_match() {
        let text = "hello world";
        let pattern = "xyz";
        let occurrences = find_all_occurrences(text, pattern, true);
        assert_eq!(occurrences, vec![]);
    }

    #[test]
    fn test_find_occurrence_to_highlight_single_match() {
        let matches = vec![(0, false)]; // One value match at index 0
        let formatted_value = "hello world";
        let search_text = "hello";
        let result = find_occurrence_to_highlight(&matches, 0, formatted_value, search_text, true);
        assert_eq!(result, Some((0, 5)));
    }

    #[test]
    fn test_find_occurrence_to_highlight_multiple_matches() {
        // Two matches in the same node: "hello" appears twice
        let matches = vec![(0, false), (1, false)]; // Two value matches
        let formatted_value = "hello world hello";
        let search_text = "hello";

        // First match should highlight first occurrence
        let result1 = find_occurrence_to_highlight(&matches, 0, formatted_value, search_text, true);
        assert_eq!(result1, Some((0, 5)));

        // Second match should highlight second occurrence
        let result2 = find_occurrence_to_highlight(&matches, 1, formatted_value, search_text, true);
        assert_eq!(result2, Some((12, 17)));
    }

    #[test]
    fn test_find_occurrence_to_highlight_with_key_matches() {
        // Mix of key and value matches
        let matches = vec![
            (0, true),  // Key match
            (1, false), // Value match (first occurrence)
            (2, false), // Value match (second occurrence)
        ];
        let formatted_value = "hello world hello";
        let search_text = "hello";

        // First value match (index 1) should highlight first occurrence
        let result1 = find_occurrence_to_highlight(&matches, 1, formatted_value, search_text, true);
        assert_eq!(result1, Some((0, 5)));

        // Second value match (index 2) should highlight second occurrence
        let result2 = find_occurrence_to_highlight(&matches, 2, formatted_value, search_text, true);
        assert_eq!(result2, Some((12, 17)));

        // Key match should return None
        let result0 = find_occurrence_to_highlight(&matches, 0, formatted_value, search_text, true);
        assert_eq!(result0, None);
    }

    #[test]
    fn test_find_occurrence_to_highlight_out_of_bounds() {
        // Test with match_index that doesn't exist in matches
        let matches = vec![(0, false), (1, false)];
        let formatted_value = "hello world";
        let search_text = "hello";

        // match_index 5 doesn't exist - should return None
        let result = find_occurrence_to_highlight(&matches, 5, formatted_value, search_text, true);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_occurrence_to_highlight_occurrence_out_of_bounds() {
        // Test when occurrence_in_node exceeds available occurrences
        let matches = vec![(0, false), (1, false), (2, false)]; // 3 matches
        let formatted_value = "hello world"; // Only 1 occurrence of "hello"
        let search_text = "hello";

        // Third match (index 2) should return None since there's only 1 occurrence
        let result = find_occurrence_to_highlight(&matches, 2, formatted_value, search_text, true);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_occurrence_to_highlight_different_paths() {
        // Simulate matches from different paths (different string values)
        // Match 0: path A, occurrence 0
        // Match 1: path A, occurrence 1
        // Match 2: path B, occurrence 0
        // Match 3: path B, occurrence 1

        // For path A with matches 0 and 1
        let path_a_matches = vec![(0, false), (1, false)];
        let path_a_value = "example one example two";
        let search_text = "example";

        // First match in path A should highlight first occurrence
        let result_a0 =
            find_occurrence_to_highlight(&path_a_matches, 0, path_a_value, search_text, true);
        assert_eq!(result_a0, Some((0, 7))); // "example" at start (bytes 0-7)

        // Second match in path A should highlight second occurrence
        // "example one " = 12 bytes, so second "example" starts at byte 12
        let result_a1 =
            find_occurrence_to_highlight(&path_a_matches, 1, path_a_value, search_text, true);
        assert_eq!(result_a1, Some((12, 19))); // "example" at position 12

        // For path B with matches 2 and 3
        let path_b_matches = vec![(2, false), (3, false)];
        let path_b_value = "example three example four";

        // First match in path B should highlight first occurrence
        let result_b0 =
            find_occurrence_to_highlight(&path_b_matches, 2, path_b_value, search_text, true);
        assert_eq!(result_b0, Some((0, 7))); // "example" at start

        // Second match in path B should highlight second occurrence
        // "example three " = 14 bytes (e-x-a-m-p-l-e-space-t-h-r-e-e-space), so second "example" starts at byte 14
        let result_b1 =
            find_occurrence_to_highlight(&path_b_matches, 3, path_b_value, search_text, true);
        assert_eq!(result_b1, Some((14, 21))); // "example" at position 14
    }

    #[test]
    fn test_find_all_occurrences_returns_char_offsets_for_unicode() {
        // Contains a Unicode curly apostrophe (U+2019) which is multi-byte in UTF-8.
        // If offsets are computed in bytes, the second match will drift.
        let text = "you’ll example then example";
        let pattern = "example";

        // Character offsets:
        // "you’ll " is 7 characters: y o u ’ l l space
        // so first "example" starts at char 7.
        // "you’ll example then " is 20 characters (7 + 7 + 6),
        // so second "example" starts at char 20.
        let occ = find_all_occurrences(text, pattern, true);
        assert_eq!(occ, vec![(7, 14), (20, 27)]);
    }
}
