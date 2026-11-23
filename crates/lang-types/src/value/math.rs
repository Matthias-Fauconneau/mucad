// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Math functions for Value

use crate::{MathOps, Ty, Value, ValueError, ValueResult};

impl MathOps for Value {
    type Error = ValueError;
    type Output = Value;

    /// Calculate the absolute value.
    fn abs(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.abs(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Calculate the square root.
    fn sqrt(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.sqrt(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Calculate the sine (expects radians or Quantity with angle units).
    fn sin(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.sin(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Calculate the cosine (expects radians or Quantity with angle units).
    fn cos(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.cos(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Calculate the tangent (expects radians or Quantity with angle units).
    fn tan(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.tan(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    fn int(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.int(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Calculate the sign of the value (-1, 0, or 1).
    fn signum(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.signum(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Round down to the nearest integer.
    fn floor(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.floor(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Round up to the nearest integer.
    fn ceil(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.ceil(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Round to the nearest integer.
    fn round(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.round(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Extract the fractional part.
    fn fract(&self) -> ValueResult {
        match self {
            Value::Number(n) => n.fract(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }
}
