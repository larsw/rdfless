// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use std::collections::HashMap;

/// Helper function to resolve a URI using prefixes
pub fn resolve_uri_with_prefixes(uri: &str, prefixes: Option<&HashMap<String, String>>) -> String {
    if let Some(prefixes) = prefixes {
        // Try to use a prefix if available
        for (prefix, iri) in prefixes {
            if uri.starts_with(iri) {
                let local_part = &uri[iri.len()..];
                return format!("{prefix}:{local_part}");
            }
        }
    }

    // No prefix found, use full URI
    format!("<{uri}>")
}

/// Get terminal height for paging decisions
pub fn get_terminal_height() -> usize {
    use terminal_size::{terminal_size, Height, Width};
    
    if let Some((Width(_), Height(height))) = terminal_size() {
        height as usize
    } else {
        24 // Default fallback
    }
}
