// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Numbers: integers and quantities, and the arithmetic between them.

use derive_more::{Display, From};
use mucad_lang_base::element::BinaryOperator;
use serde::{Deserialize, Serialize};

use crate::{
    Integer, MathOps, Quantity, QuantityType, Scalar, Type, Value, ValueError, ValueResult,
};

/// A number: an integer or a quantity, e.g. `3`, `1.5` or `10mm`.
///
/// An integer is converted into a scalar, if it is combined with a quantity, e.g. `1 + 0.5`.
#[derive(Clone, Debug, Display, Hash, PartialEq, From, Serialize, Deserialize)]
pub enum Number {
    /// A quantity value.
    Quantity(Quantity),
    /// An integer value.
    Integer(Integer),
}

impl Number {
    /// The number as quantity, an integer as scalar.
    pub fn to_quantity(&self) -> Quantity {
        match self {
            Number::Quantity(quantity) => quantity.clone(),
            Number::Integer(integer) => Quantity::from(*integer),
        }
    }

    /// Compare two numbers, e.g. with `<`.
    pub fn cmp(&self, op: BinaryOperator, rhs: &Self) -> ValueResult {
        match (self, rhs) {
            (Number::Integer(lhs), Number::Integer(rhs)) => Ok(Value::Bool(match op {
                BinaryOperator::Equal => lhs == rhs,
                BinaryOperator::NotEqual => lhs != rhs,
                BinaryOperator::GreaterThan => lhs > rhs,
                BinaryOperator::GreaterEqual => lhs >= rhs,
                BinaryOperator::LessThan => lhs < rhs,
                BinaryOperator::LessEqual => lhs <= rhs,
                BinaryOperator::Near => lhs == rhs, // Integers are exact
                _ => unreachable!(),
            })),
            (lhs, rhs) => lhs.to_quantity().cmp(op, &rhs.to_quantity()),
        }
    }

    /// Calculate the power of two numbers, if possible.
    pub fn pow(&self, rhs: &Self) -> ValueResult {
        match (self, rhs) {
            (Number::Quantity(lhs), Number::Quantity(rhs)) => Ok(lhs.pow(rhs).into()),
            (Number::Quantity(lhs), Number::Integer(rhs)) => Ok(lhs.pow_int(rhs).into()),
            (Number::Integer(_), Number::Integer(_)) => todo!(),
            _ => Err(ValueError::InvalidOperator("^".to_string())),
        }
    }
}

impl crate::ty::Ty for Number {
    fn ty(&self) -> Type {
        match self {
            Number::Integer(_) => Type::Integer,
            Number::Quantity(quantity) => quantity.ty(),
        }
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Number::Integer(lhs), Number::Integer(rhs)) => lhs.partial_cmp(rhs),
            (Number::Quantity(lhs), Number::Quantity(rhs)) => lhs.partial_cmp(rhs),
            (
                Number::Quantity(Quantity {
                    value,
                    quantity_type: QuantityType::Scalar,
                    ..
                }),
                Number::Integer(rhs),
            ) => value.partial_cmp(&Scalar::from(*rhs)),
            _ => None,
        }
    }
}

impl std::ops::Neg for Number {
    type Output = Number;

    fn neg(self) -> Self::Output {
        match self {
            Number::Integer(integer) => Number::Integer(-integer),
            Number::Quantity(quantity) => Number::Quantity(-quantity),
        }
    }
}

/// An arithmetic operator for numbers, whose result of two integers is an integer.
macro_rules! impl_op {
    ($trait:ident, $fn:ident, $op:tt) => {
        impl std::ops::$trait for Number {
            type Output = ValueResult;

            fn $fn(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Number::Integer(lhs), Number::Integer(rhs)) => Ok(Value::from(lhs $op rhs)),
                    (Number::Integer(lhs), Number::Quantity(rhs)) => lhs $op rhs,
                    (Number::Quantity(lhs), Number::Integer(rhs)) => lhs $op rhs,
                    (Number::Quantity(lhs), Number::Quantity(rhs)) => lhs $op rhs,
                }
            }
        }
    };
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);

/// The division, whose result of two integers is a scalar.
impl std::ops::Div for Number {
    type Output = ValueResult;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Integer(lhs), Number::Integer(rhs)) => {
                Ok((Scalar::from(lhs) / Scalar::from(rhs)).into())
            }
            (Number::Quantity(lhs), Number::Integer(rhs)) => lhs / rhs,
            (Number::Integer(lhs), Number::Quantity(rhs)) => lhs / rhs,
            (Number::Quantity(lhs), Number::Quantity(rhs)) => lhs / rhs,
        }
    }
}

/// A math function, whose result of an integer is an integer, e.g. of `abs`.
macro_rules! integer_math {
    ($($fn:ident: $integer:expr),* $(,)?) => {
        $(
            fn $fn(&self) -> ValueResult {
                match self {
                    Number::Integer(i) => Ok(Value::from($integer(*i))),
                    Number::Quantity(q) => q.$fn(),
                }
            }
        )*
    };
}

/// A math function, which converts an integer into a scalar, e.g. of `sqrt`.
macro_rules! scalar_math {
    ($($fn:ident),* $(,)?) => {
        $(
            fn $fn(&self) -> ValueResult {
                self.to_quantity().$fn()
            }
        )*
    };
}

impl MathOps for Number {
    type Error = ValueError;
    type Output = Value;

    integer_math!(
        abs: Integer::abs,
        signum: Integer::signum,
        int: std::convert::identity,
        floor: std::convert::identity,
        ceil: std::convert::identity,
        round: std::convert::identity,
        fract: |_| Integer::ZERO,
    );
    scalar_math!(sqrt, sin, cos, tan);
}
