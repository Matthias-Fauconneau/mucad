// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{Diagnostics, IssueList, Source};
use mucad_lang_parse::{Ast, ParseIssue, ast, parse};

mod expression;
mod extras;
mod layout;
mod literal;
mod node;
mod statement;
mod ty;

pub(crate) use crate::{
    layout::Column,
    node::{Break, Node},
};

/// The settings of the formatter.
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// The maximal width of a line, which is exceeded only by code without a possible line
    /// break, e.g. a long string.
    pub max_width: usize,
    /// The width of an indentation.
    pub indent_width: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            max_width: 100,
            indent_width: 4,
        }
    }
}

/// The nodes of formatted code, which have the structure of the code, independent of the
/// [`FormatConfig`], which only lays them out, see [`Node::render`].
pub(crate) trait Format {
    fn format(&self) -> Node;
}

impl<T> Format for T
where
    T: Into<Node> + Clone,
{
    fn format(&self) -> Node {
        self.clone().into()
    }
}

impl<T: Format> Format for Option<T> {
    fn format(&self) -> Node {
        match self {
            Some(inner) => inner.format(),
            None => Node::Nil,
        }
    }
}

impl Format for ast::Identifier {
    fn format(&self) -> Node {
        self.name.clone().into()
    }
}

impl Format for ast::Comment {
    fn format(&self) -> Node {
        match &self.inner {
            ast::CommentInner::SingleLine(line) => {
                node!(Node::Column(Column::Comment) Break::Space line.clone() Break::Hard)
            }
            ast::CommentInner::MultiLine(line) => {
                node!(Node::Column(Column::Comment) Break::Space "/* " line.clone() " */" Break::Space)
            }
        }
    }
}

impl Format for ast::DocBlock {
    fn format(&self) -> Node {
        self.lines
            .iter()
            .map(|line| node!(line.clone() Break::Hard))
            .collect::<Vec<_>>()
            .into()
    }
}

impl Format for ast::Source {
    fn format(&self) -> Node {
        node!(self.statements.extras => self.statements)
    }
}

impl Format for Ast {
    fn format(&self) -> Node {
        self.tree().format()
    }
}

/// The formatted nodes one after another, e.g. `node!(self.id " = " self.expr)`, between the
/// leading and the trailing extras of an item with `node!(self.extras => …)`.
#[macro_export]
macro_rules! node {
    ($extras:expr => $($node:expr)*) => {
        $crate::extras::with_extras(&$extras, node!($($node)*))
    };
    ($($node:expr)*) => {{
        // The method call dereferences, e.g. a `Box` of an expression.
        #[allow(unused_imports)]
        use $crate::Format as _;
        $crate::Node::from(vec![$( ($node).format() ),*])
    }};
}

/// Format µcad a parsed abstract syntax tree into a string
pub fn format_ast(ast: &Ast, config: &FormatConfig) -> String {
    ast.format().render(config)
}

/// A formatted source and AST.
pub struct Formatted {
    pub source: Source,
    pub ast: Ast,
    pub formatted: bool,
    pub issues: IssueList<ParseIssue>,
}

/// An error of formatting code.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// The code cannot be parsed, so it is not formatted.
    #[error("the code cannot be parsed")]
    Parse(IssueList<ParseIssue>),
    /// The formatted code cannot be parsed, which is an error of the formatter.
    #[error("the formatted code cannot be parsed, which is an error of the formatter")]
    InvalidFormat {
        /// The formatted code, which the issues refer to, boxed for a small error.
        formatted: Box<Source>,
        /// The issues of parsing the formatted code.
        issues: IssueList<ParseIssue>,
    },
}

/// Format an AST into a new AST and a new Source
///
/// The formatted code is parsed again, it fails, if the formatter writes invalid code.
pub fn format(source: impl Into<Source>, config: &FormatConfig) -> Result<Formatted, FormatError> {
    let original_source = source.into();
    let (ast, _) = parse(&original_source).map_err(FormatError::Parse)?;
    let code = format_ast(&ast, config);
    let source = Source::new(original_source.location.clone(), code);
    match parse(&source) {
        Ok((ast, issues)) => Ok(Formatted {
            formatted: source != original_source,
            ast,
            source,
            issues,
        }),
        Err(issues) => Err(FormatError::InvalidFormat {
            formatted: Box::new(source),
            issues,
        }),
    }
}

/// High-level API to format a &str containing µcad source code.
pub fn format_code(code: &str, config: &FormatConfig) -> Result<String, Diagnostics> {
    let source = Source::from(code);
    let ast = parse(&source)?.0;
    Ok(format_ast(&ast, config))
}
