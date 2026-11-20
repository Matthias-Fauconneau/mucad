// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_parse::ast;

use crate::{Format, FormatConfig, Node, node, node::Break};

/// The comments and line breaks of extras.
///
/// A line break of the source is a [`Break::Soft`], so the formatter decides about it, but it
/// is a [`Break::Hard`] in front of a comment, which stays on its own line. Blank lines are a
/// [`Break::Paragraph`].
fn format_extras(extras: &[ast::ItemExtra], f: &FormatConfig) -> Node {
    extras
        .iter()
        .enumerate()
        .map(|(i, extra)| match extra {
            ast::ItemExtra::Comment(comment) => comment.format(f),
            ast::ItemExtra::Whitespace(ast::Whitespace(ws)) => {
                let before_comment = matches!(extras.get(i + 1), Some(ast::ItemExtra::Comment(_)));
                match ws.matches('\n').count() {
                    0 => Node::Nil,
                    1 if before_comment => Break::Hard.into(),
                    1 => Break::Soft.into(),
                    _ => Break::Paragraph.into(),
                }
            }
            _ => unimplemented!(),
        })
        .collect::<Vec<_>>()
        .into()
}

impl Format for ast::LeadingExtras {
    fn format(&self, f: &FormatConfig) -> Node {
        format_extras(&self.0, f)
    }
}

impl Format for ast::TrailingExtras {
    fn format(&self, f: &FormatConfig) -> Node {
        format_extras(&self.0, f)
    }
}

pub(crate) fn with_extras(
    extras: &ast::ItemExtras,
    f: &FormatConfig,
    node: impl Into<Node>,
) -> Node {
    node!(f =>
        extras.leading
        node.into()
        extras.trailing
    )
}
