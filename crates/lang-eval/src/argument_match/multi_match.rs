// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Match arguments with lists to multiple argument combinations.

use mucad_lang_base::Identifier;
use mucad_lang_types::{ArgumentValueList, Arguments, Tuple, Ty, Value};

use crate::{ArgumentMatch, ArgumentMatchError};

/// Match arguments for a function call, expanding at most one list argument.
///
/// The boolean indicates whether a list was expanded. An empty list still counts as an
/// expansion and therefore produces an empty result list.
pub(crate) fn argument_single_match(
    matcher: &(impl ArgumentMatch + ?Sized),
    arguments: &ArgumentValueList,
) -> Result<(bool, Vec<Arguments>), ArgumentMatchError> {
    let base_args = matcher.argument_match(arguments)?;
    let Some(params) = matcher.call_signature().parameters else {
        return Ok((false, vec![base_args]));
    };

    let mut expansion: Option<(Identifier, Vec<Value>)> = None;
    for (id, value) in &base_args.0.named {
        let Some(expected) = params
            .iter()
            .find(|(param_id, _)| param_id == id)
            .map(|(_, ty)| ty)
        else {
            continue;
        };

        let Value::List(elements) = value else {
            continue;
        };

        let expands = value.ty().matches_list_type(expected)
            || (elements.is_empty()
                && !matches!(
                    expected,
                    mucad_lang_types::Type::List(_) | mucad_lang_types::Type::Any
                ));
        if !expands {
            continue;
        }

        if expansion.is_some() {
            return Err(ArgumentMatchError::MultipleListArguments {
                arguments: arguments.to_string(),
            });
        }

        expansion = Some((id.clone(), elements.iter().cloned().collect()));
    }

    let Some((id, values)) = expansion else {
        return Ok((false, vec![base_args]));
    };

    let matches = values
        .into_iter()
        .map(|value| {
            let mut args = base_args.clone();
            let (_, arg_value) = args
                .0
                .named
                .iter_mut()
                .find(|(arg_id, _)| arg_id == &id)
                .expect("expanded argument is present");
            *arg_value = value;
            args
        })
        .collect();

    Ok((true, matches))
}

/// Match `arguments`, a list argument for a non-list parameter results in one match per element.
///
/// Returns the cartesian product of all such lists.
pub(super) fn argument_multi_match(
    matcher: &(impl ArgumentMatch + ?Sized),
    arguments: &ArgumentValueList,
) -> Result<Vec<Arguments>, ArgumentMatchError> {
    // Step 1: Bind base arguments and resolve default values
    let base_args = matcher.argument_match(arguments)?;

    let call_signature = matcher.call_signature();
    let Some(params) = call_signature.parameters else {
        return Err(ArgumentMatchError::MultiMatchNotAllowedForVariadic {
            arguments: arguments.to_string(),
        });
    };

    // Step 2: Identify which arguments need vectorization
    let resolved_lists: Vec<(Identifier, Vec<Value>)> = base_args
        .0
        .named
        .into_iter()
        .map(|(id, val)| {
            let expected_ty = params
                .iter()
                .find(|(p_id, _)| p_id == &id)
                .map(|(_, ty)| ty);
            match (expected_ty, &val) {
                (Some(expected), Value::List(elements)) if val.ty().matches_list_type(expected) => {
                    let elements = elements.to_vec();
                    (id, elements)
                }
                _ => (id, vec![val]),
            }
        })
        .collect();

    // Step 3: Compute Cartesian product across all argument value lists
    let mut combinations: Vec<Vec<(Identifier, Value)>> = vec![vec![]];
    for (id, values) in resolved_lists {
        let mut next_combinations = Vec::new();
        for combination in combinations {
            for val in &values {
                let mut extended = combination.clone();
                extended.push((id.clone(), val.clone()));
                next_combinations.push(extended);
            }
        }
        combinations = next_combinations;
    }

    // Step 4: Convert combinations into Arguments
    Ok(combinations
        .into_iter()
        .map(|named| {
            Tuple {
                positional: vec![],
                named,
            }
            .into()
        })
        .collect())
}
