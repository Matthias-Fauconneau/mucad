// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_parse::ast;

use crate::{Format, Node, node, node::Break};

/// The comments and line breaks of extras.
///
/// A line break of the source is a [`Break::Space`], so the formatter decides about it, but it
/// is a [`Break::Hard`] in front of a comment, which stays on its own line. Blank lines are a
/// [`Break::Paragraph`].
fn format_extras(extras: &[ast::ItemExtra]) -> Node {
    extras
        .iter()
        .enumerate()
        .map(|(i, extra)| match extra {
            ast::ItemExtra::Comment(comment) => comment.format(),
            ast::ItemExtra::Whitespace(ast::Whitespace(ws)) => {
                let before_comment = matches!(extras.get(i + 1), Some(ast::ItemExtra::Comment(_)));
                match ws.matches('\n').count() {
                    0 => Node::Nil,
                    1 if before_comment => Break::Hard.into(),
                    1 => Break::Space.into(),
                    _ => Break::Paragraph.into(),
                }
            }
            _ => unimplemented!(),
        })
        .collect::<Vec<_>>()
        .into()
}

impl Format for ast::LeadingExtras {
    fn format(&self) -> Node {
        format_extras(&self.0)
    }
}

impl Format for ast::TrailingExtras {
    fn format(&self) -> Node {
        format_extras(&self.0)
    }
}

/// A node between the leading and the trailing extras of its item.
pub(crate) fn with_extras(extras: &ast::ItemExtras, node: Node) -> Node {
    node!(extras.leading node extras.trailing)
}
