// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::{config, formatter, types::ArgsConfig, utils};
use anyhow::Result;
use std::io::IsTerminal;

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

/// Process input with automatic paging detection
pub fn process_with_auto_pager<A: ArgsConfig>(
    triples: &[crate::types::OwnedTriple],
    prefixes: &std::collections::HashMap<String, String>,
    args: &A,
    config: &config::Config,
) -> Result<()> {
    let colors = &args.get_colors(config);
    let should_expand = args.expand(config);
    let estimated_lines = formatter::estimate_output_lines(triples, prefixes, should_expand);

    // Determine if we should use paging
    let use_paging = should_use_pager(args, config, estimated_lines);

    if use_paging && std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        // Use pager
        let mut output = Vec::new();
        formatter::render_output(triples, prefixes, should_expand, colors, &mut output)?;
        let output_str = String::from_utf8(output)?;

        let pager = minus::Pager::new();
        pager.set_text(output_str)?;
        minus::page_all(pager)?;
    } else {
        // Direct output
        formatter::render_output(
            triples,
            prefixes,
            should_expand,
            colors,
            &mut std::io::stdout(),
        )?;
    }

    Ok(())
}
