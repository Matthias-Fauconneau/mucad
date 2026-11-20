// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Break, BreakMode, Format, FormatConfig, Node, node, statement::statements_and_tail};

use mucad_lang_parse::{ast, token::escape_char};

impl Format for ast::BinaryOperator {
    fn format(&self, _: &FormatConfig) -> Node {
        self.to_string().into()
    }
}

impl Format for ast::UnaryOperator {
    fn format(&self, _: &FormatConfig) -> Node {
        self.to_string().into()
    }
}

impl Format for ast::Body {
    fn format(&self, f: &FormatConfig) -> Node {
        format_body(self, true, f)
    }
}

/// A body, which is on one line, if it only has a tail without line breaks and `inline` is
/// set, e.g. `{ a }`.
///
/// Otherwise, the statements start a line, behind a comment on the line of the `{`.
fn format_body(body: &ast::Body, inline: bool, f: &FormatConfig) -> Node {
    let list = &body.statements;

    if list.statements.is_empty() {
        match &list.tail {
            None => {
                let extras = node!(f, list.extras => Node::Nil);
                // Without comments, an empty body is `{}`.
                return match extras {
                    Node::Nil | Node::Break(_) if !extras.has_line_break() => "{}".into(),
                    extras => node!("{" Node::indent(f.indent_width, extras) "}"),
                };
            }
            Some(tail) if inline => {
                let tail = node!(f, list.extras => tail.format(f));
                if !tail.has_line_break() {
                    return node!("{ " tail " }");
                }
            }
            Some(_) => {}
        }
    }

    node!(
        "{"
        Node::indent(f.indent_width, node!(f =>
            list.extras.leading
            Break::Hard
            statements_and_tail(list, f)
            list.extras.trailing
        ))
        Break::Hard
        "}"
    )
}

impl Format for ast::Expression {
    fn format(&self, f: &FormatConfig) -> Node {
        match &self {
            ast::Expression::Literal(literal) => literal.format(f),
            ast::Expression::Bracketed(bracket, _) => node!('(' bracket.format(f) ')'),
            ast::Expression::Tuple(tuple_expression) => tuple_expression.format(f),
            ast::Expression::Range(range_expr) => range_expr.format(f),
            ast::Expression::List(list_expr) => list_expr.format(f),
            ast::Expression::String(format_string) => format_string.format(f),
            ast::Expression::SymbolPath(qualified_name) => qualified_name.format(f),
            ast::Expression::Marker(identifier) => format!("@{}", identifier.name).into(),
            ast::Expression::BinaryOperation(binary_operation) => binary_operation.format(f),
            ast::Expression::UnaryOperation(unary_operation) => unary_operation.format(f),
            ast::Expression::Body(body) => body.format(f),
            ast::Expression::Call(call) => call.format(f),
            ast::Expression::ElementAccess(element_access) => element_access.format(f),
            ast::Expression::If(i) => i.format(f),
            ast::Expression::Error(_) => Node::Nil,
        }
    }
}

impl Format for ast::StringPart {
    fn format(&self, f: &FormatConfig) -> Node {
        match &self {
            ast::StringPart::Char(string_character) => string_character.format(f),
            ast::StringPart::Content(string_literal) =>
            // Simply clone the content of the string literal, because we do not want to have quotes '"'
            {
                string_literal.content.clone().into()
            }
            ast::StringPart::Expression(string_expression) => string_expression.format(f),
        }
    }
}

impl Format for ast::StringCharacter {
    fn format<'a>(&self, _: &FormatConfig) -> Node {
        // An escaped character, e.g. `\/`, which is no escape sequence by `escape_char`.
        match escape_char(self.character) {
            Some(escaped) => escaped.into(),
            None => format!("\\{}", self.character).into(),
        }
    }
}

impl Format for ast::StringExpression {
    fn format<'a>(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            '{' self.expr self.specification '}'
        )
    }
}

impl Format for ast::StringFormatSpecification {
    fn format<'a>(&self, _f: &FormatConfig) -> Node {
        match (&self.width, &self.precision) {
            (Some(Ok(width)), Some(Ok(precision))) => format!(":0{width}.{precision}").into(),
            (None, Some(Ok(precision))) => format!(":.{precision}").into(),
            (Some(Ok(width)), None) => format!(":0{width}").into(),
            _ => Node::Nil,
        }
    }
}

impl Format for ast::FormatString {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            '"'
            self.parts
                .iter()
                .map(|part| part.format(f))
                .collect::<Vec<_>>()
            '"'
        )
    }
}

impl Format for ast::TupleItem {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            match &self.id {
                Some(name) => node!(f => name " = " self.expr),
                None => node!(f => self.expr)
            }
        )
    }
}

impl Format for ast::TupleExpression {
    fn format(&self, f: &FormatConfig) -> Node {
        let nodes: Vec<Node> = self.values.iter().map(|item| item.format(f)).collect();
        let break_mode = BreakMode::from_layout(&nodes, 4, f);
        node!(f, self.extras =>
            '(' Node::list(nodes, ',', break_mode) ')'
        )
    }
}

impl Format for ast::ListItem {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras => self.expr)
    }
}

impl Format for ast::RangeExpression {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            '[' self.start ".." self.end ']' self.unit
        )
    }
}

impl Format for ast::ListExpression {
    fn format(&self, f: &FormatConfig) -> Node {
        let nodes: Vec<Node> = self.items.iter().map(|item| item.format(f)).collect();
        let break_mode = BreakMode::from_layout(&nodes, 0, f);
        node!(f, self.extras =>
            '[' Node::list(nodes, ',', break_mode) ']' self.unit
        )
    }
}

impl Format for ast::SymbolPath {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            if self.prefix.is_some() { "::" } else { "" }
            Node::hlist(self.parts.iter().map(|identifier| identifier.format(f)), "::")
        )
    }
}

impl Format for ast::BinaryOperation {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f => self.lhs Break::Soft self.op Break::Soft self.rhs)
    }
}

impl Format for ast::UnaryOperation {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f => self.op self.rhs)
    }
}

impl Format for ast::Argument {
    fn format(&self, f: &FormatConfig) -> Node {
        match self {
            ast::Argument::Unnamed(arg) => arg.format(f),
            ast::Argument::Named(arg) => arg.format(f),
        }
    }
}

impl Format for ast::UnnamedArgument {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras => self.expr)
    }
}

impl Format for ast::NamedArgument {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras => self.id " = " self.expr)
    }
}

impl Format for ast::ArgumentList {
    fn format(&self, f: &FormatConfig) -> Node {
        let nodes: Vec<Node> = self.arguments.iter().map(|item| item.format(f)).collect();
        let break_mode = BreakMode::from_layout(&nodes, 4, f);

        node!('(' node!(f, self.extras => Node::list(nodes, ',', break_mode)) ')')
    }
}

impl Format for ast::Call {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras => self.path self.arguments)
    }
}

impl Format for ast::ElementInner {
    fn format(&self, f: &FormatConfig) -> Node {
        use ast::ElementInner::*;
        match &self {
            Attribute(identifier) => node!(f => '#' identifier),
            Tuple(identifier) => node!(f => '.' identifier),
            Method(call) => node!(f => '.' call),
            ListElement(expression) => node!(f => '[' expression ']'),
        }
    }
}

impl Format for ast::Element {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras => self.inner)
    }
}

/// A chain with more than two method calls, or a comment, has each method call on its own
/// line, e.g.:
///
/// ```mu
/// Circle(r = 5mm)
///     .extrude(5mm)
///     .translate(z = 2mm)
///     .center();
/// ```
///
/// The method calls are indented, unless the chain starts with a body, e.g.:
///
/// ```mu
/// {
///     Circle(r = 5mm);
/// }
/// .extrude(5mm)
/// .translate(z = 2mm)
/// .center();
/// ```
impl Format for ast::ElementAccess {
    fn format(&self, f: &FormatConfig) -> Node {
        /// The most method calls on one line.
        const MAX_METHODS: usize = 2;

        let expr = self.expr.format(f);
        let elements: Vec<Node> = self
            .element_chain
            .iter()
            .map(|element| element.format(f))
            .collect();

        let methods = self
            .element_chain
            .iter()
            .filter(|element| matches!(element.inner, ast::ElementInner::Method(_)))
            .count();
        let commented =
            expr.ends_with_line_break() || elements.iter().any(Node::ends_with_line_break);
        if methods <= MAX_METHODS && !commented {
            return node!(expr elements);
        }

        let chain: Vec<Node> = self
            .element_chain
            .iter()
            .zip(elements)
            .map(|(element, node)| match element.inner {
                ast::ElementInner::Method(_) => node!(Break::Hard node),
                _ => node,
            })
            .collect();
        match self.expr.as_ref() {
            ast::Expression::Body(_) => node!(expr chain),
            _ => node!(expr Node::indent(f.indent_width, chain)),
        }
    }
}

impl Format for ast::If {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            "if " self.condition ' ' format_body(&self.body, false, f)
            match &self.else_body {
                Some(else_body) => node!(f => Break::Soft "else " format_body(else_body, false, f)),
                None => match &self.next_if {
                    Some(_) => Node::Nil,
                    None => Break::Hard.into()
                }
            }
            match &self.next_if {
                Some(next_if) => node!(f => Break::Soft "else " next_if),
                None => Node::Nil
            }
        )
    }
}
