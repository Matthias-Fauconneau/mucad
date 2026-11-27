// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Match the arguments of a call to parameters.

mod matcher;

use miette::Diagnostic;
use mucad_lang_base::{Identifier, IdentifierList};
use mucad_lang_resolve::library::symbol::{
    Function, ParameterList, function::FunctionSignature, workbench::Init,
};
use mucad_lang_types::{ArgumentValueList, Arguments, CallSignature, List, Tuple, Ty, Type, Value};
use thiserror::Error;

use matcher::Matcher;

#[derive(Error, Debug, Diagnostic)]
pub enum ArgumentMatchError {
    /// Missing arguments
    #[error("Missing arguments: {0}")]
    MissingArguments(IdentifierList),

    /// Unexpected arguments
    #[error("Unexpected arguments: {0}")]
    UnexpectedArguments(IdentifierList),

    /// Positional arguments, which match no parameter by type.
    #[error("Unexpected positional arguments: {0}")]
    UnexpectedPositionalArguments(String),

    /// An abbreviated argument matches several parameters.
    #[error("Argument `{argument}` is ambiguous: {parameters}")]
    #[diagnostic(help("Use the full parameter name."))]
    AmbiguousArgument {
        /// The abbreviated argument, e.g. `f`.
        argument: Identifier,
        /// The parameters starting like the argument, e.g. `foo` and `foz`.
        parameters: IdentifierList,
    },

    /// A parameter is given more than once, e.g. as `r` and `radius`.
    #[error("Argument `{0}` is given more than once")]
    DuplicateArgument(Identifier),
}

/// The priority of a match of arguments to parameters, higher is better.
///
/// It chooses between overloaded initializers, e.g. of `std::ops::rotate`.
/// Initializers with the same priority are chosen by their definition order.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MatchPriority {
    /// The number of named arguments and inline identifiers, which match a parameter by name,
    /// e.g. `x = 1` matches `init(x, y, z)` before `init(xyz)`.
    pub exact_names: usize,
    /// The number of named arguments and inline identifiers, which match a parameter by short name,
    /// e.g. `r` in `Circle(r)` matches `radius` before `diameter` by type.
    pub short_names: usize,
}

/// The arguments of the calls of a match, one call per element of an expanded list.
#[derive(Debug, Clone, PartialEq)]
pub struct Matches {
    pub arguments: Vec<Arguments>,
    /// Whether a list was expanded, even an empty one, which results in no call.
    pub expanded: bool,
}

impl Matches {
    /// The arguments of one call without expansion.
    fn single(arguments: Arguments) -> Self {
        Self {
            arguments: vec![arguments],
            expanded: false,
        }
    }

    /// The value of the calls with the arguments, which is a list of their values, if a list
    /// was expanded.
    pub fn try_value<E>(self, call: impl FnMut(Arguments) -> Result<Value, E>) -> Result<Value, E> {
        let mut values = self.arguments.into_iter().map(call);
        match self.expanded {
            true => Ok(Value::list(values.collect::<Result<Vec<_>, _>>()?)),
            false => values.next().transpose().map(Option::unwrap_or_default),
        }
    }
}

pub trait ArgumentMatch {
    fn call_signature(&self) -> CallSignature;

    fn default_values(&self) -> Tuple {
        Tuple::default()
    }

    /// Match `arguments` to the parameters, and expand list arguments for parameters of their
    /// element type into one call per combination of their elements, the cartesian product.
    ///
    /// An empty list for a parameter, which is no list, still counts as an expansion and
    /// therefore produces no call. The arguments of a variadic call signature are not expanded.
    fn match_arguments(
        &self,
        arguments: &ArgumentValueList,
    ) -> Result<Matches, ArgumentMatchError> {
        let arguments = self.argument_match(arguments)?;
        let Some(parameters) = self.call_signature().parameters else {
            return Ok(Matches::single(arguments));
        };

        // A list expands, if its elements have the type of the parameter, or if it is empty
        // and the parameter is no list.
        let expands = |list: &List, expected: &Type| {
            list.ty().matches_list_type(expected)
                || (list.is_empty() && !matches!(expected, Type::List(_) | Type::Any))
        };

        // The values of each argument, which are the elements of an expanded list.
        let mut expanded = false;
        let values: Vec<(Identifier, Vec<Value>)> = arguments
            .0
            .named
            .into_iter()
            .map(|(id, value)| {
                let expected = parameters.iter().find(|(param, _)| param == &id);
                match value {
                    Value::List(list) if expected.is_some_and(|(_, ty)| expands(&list, ty)) => {
                        expanded = true;
                        (id, list.iter().cloned().collect())
                    }
                    value => (id, vec![value]),
                }
            })
            .collect();

        // The cartesian product of the values.
        let mut combinations = vec![Vec::new()];
        for (id, values) in values {
            combinations = combinations
                .iter()
                .flat_map(|combination| {
                    values.iter().map(|value| {
                        let item = (id.clone(), value.clone());
                        combination
                            .iter()
                            .cloned()
                            .chain([item])
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
        }

        Ok(Matches {
            arguments: combinations
                .into_iter()
                .map(|named| {
                    Arguments::from(Tuple {
                        positional: vec![],
                        named,
                    })
                })
                .collect(),
            expanded,
        })
    }

    /// Match `arguments` to the parameters of the call signature.
    ///
    /// Named arguments are matched by name or by an abbreviation (e.g. `r` for `radius`),
    /// positional arguments by type. Parameters without argument get their default value.
    fn argument_match(
        &self,
        arguments: &ArgumentValueList,
    ) -> Result<Arguments, ArgumentMatchError> {
        match self.call_signature().parameters {
            Some(parameters) => {
                Matcher::new(&parameters, &self.default_values()).match_arguments(arguments)
            }
            // A variadic function simply returns the arguments as tuple,
            // arguments with inline identifiers are named.
            None => Ok(Arguments::from(Tuple {
                named: arguments
                    .iter()
                    .filter_map(|arg| Some((arg.name.id()?.clone(), arg.value.clone())))
                    .collect(),
                positional: arguments
                    .iter()
                    .filter(|arg| arg.name.id().is_none())
                    .map(|arg| arg.value.clone())
                    .collect(),
            })),
        }
    }

    /// The priority of the match of `arguments`, or `Err` if they don't match.
    fn match_priority(&self, arguments: &ArgumentValueList) -> Result<MatchPriority, ArgumentMatchError> {
        match self.call_signature().parameters {
            Some(parameters) => Matcher::new(&parameters, &self.default_values())
                .match_priority(arguments),
            None => Ok(MatchPriority::default()),
        }
    }
}

impl ArgumentMatch for ParameterList {
    /// Return default values for this parameters, assuming all constant expression have been folded into values.
    fn default_values(&self) -> Tuple {
        use mucad_lang_resolve::library::symbol::ExprSpec;
        Tuple::from_iter(self.parameters.iter().filter_map(|param| {
            param
                .default_value
                .as_ref()
                .and_then(|expr| expr.value())
                .map(|value| (param.id.clone(), value.clone()))
        }))
    }

    /// Parameters call signature
    fn call_signature(&self) -> CallSignature {
        use mucad_lang_types::Ty;
        CallSignature::new(
            self.iter()
                .map(|param| (param.id.clone(), param.ty()))
                .collect::<Vec<_>>(),
        )
    }
}

impl ArgumentMatch for Init {
    fn call_signature(&self) -> CallSignature {
        self.parameters.call_signature()
    }

    fn default_values(&self) -> Tuple {
        self.parameters.default_values()
    }
}

impl ArgumentMatch for FunctionSignature {
    fn call_signature(&self) -> CallSignature {
        self.parameters.call_signature()
    }

    fn default_values(&self) -> Tuple {
        self.parameters.default_values()
    }
}

impl ArgumentMatch for Function {
    fn call_signature(&self) -> CallSignature {
        self.signature.call_signature()
    }

    fn default_values(&self) -> Tuple {
        self.signature.default_values()
    }
}

impl ArgumentMatch for mucad_builtin::BuiltinFunction {
    fn call_signature(&self) -> CallSignature {
        self.ty().call_ty.clone()
    }

    fn default_values(&self) -> Tuple {
        self.default_values()
    }
}

impl ArgumentMatch for mucad_builtin::BuiltinPrimitive {
    fn call_signature(&self) -> CallSignature {
        self.ty().call_ty.clone()
    }
}

impl ArgumentMatch for mucad_builtin::BuiltinOperation {
    fn call_signature(&self) -> CallSignature {
        (self.ty)().call_ty.clone()
    }
}
