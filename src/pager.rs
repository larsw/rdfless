// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::{config, types::ArgsConfig, utils};

/// Determine if paging should be used based on content length
pub fn should_use_pager<A: ArgsConfig>(
    args: &A,
    config: &config::Config,
    estimated_lines: usize,
) -> bool {
    // If user explicitly disabled paging, respect that
    if args.no_pager_explicit() {
        return false;
    }

    // If user explicitly enabled paging, respect that
    if args.use_pager(config) {
        return true;
    }

    // Check if auto-paging is enabled
    if !config.output.auto_pager {
        return false;
    }

    // Determine threshold
    let threshold = if config.output.auto_pager_threshold > 0 {
        config.output.auto_pager_threshold
    } else {
        utils::get_terminal_height().saturating_sub(2) // Leave some space for prompt
    };

    estimated_lines > threshold
}
