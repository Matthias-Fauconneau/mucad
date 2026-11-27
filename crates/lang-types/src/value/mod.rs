// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evaluation entities.
//!
//! Every evaluation of any *symbol* leads to a [`Value`] which then might continued
//! to process or ends up as the overall evaluation result.

mod list;
mod literal;
mod math;
mod matrix;
mod number;
pub mod ops;
mod quantity;
mod tuple;
mod value_access;
mod value_error;
mod value_list;

use std::rc::Rc;

use fixed::traits::ToFixed;
pub use list::*;
pub use literal::Literal;
pub use matrix::*;
use mucad_lang_base::{CompactString, DisplayOneLine, ToCompactString};
pub use number::*;
pub use quantity::*;
pub use tuple::*;
pub use value_access::*;
pub use value_error::*;
pub use value_list::*;

use crate::{
    Angle, Color, Integer, Length, Model, ModelTree, QuantityType, Scalar, Type, Unit, Vec2, Vec3,
};

use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

pub type ValueResult<Type = Value> = std::result::Result<Type, ValueError>;

/// Any value in a µcad model.
#[derive(Clone, Debug, Display, Hash, Default, PartialEq, From, Serialize, Deserialize)]
pub enum Value {
    /// A None Value.
    #[default]
    #[display("<NO VALUE>")]
    None,
    /// A number.
    Number(Number),
    /// A boolean value.
    Bool(bool),
    /// A string value.
    String(CompactString),
    /// A list of values with a common type.
    List(Rc<List>),
    /// A tuple of named items.
    Tuple(Rc<Tuple>),
    /// A matrix.
    Matrix(Rc<Matrix>),
    /// A model tree
    Model(Rc<ModelTree>),
}

impl DisplayOneLine for Value {}

/// Builder methods, mainly for testing
impl Value {
    /// Create angle from scalar in  degrees
    pub fn deg(angle: crate::math::ScalarF) -> Self {
        Quantity::new(angle.to_radians().to_fixed(), QuantityType::Angle)
            .with_unit(Unit::Deg)
            .into()
    }

    /// Create a lange from scalar in mm
    pub fn mm(l: crate::math::ScalarF) -> Self {
        Quantity::new(l.to_fixed(), QuantityType::Length)
            .with_unit(Unit::Millimeter)
            .into()
    }

    pub fn list<T: Into<Value>>(iter: impl IntoIterator<Item = T>) -> Self {
        Self::List(Rc::new(List::from_values(
            iter.into_iter().map(|t| t.into()).collect(),
        )))
    }
}

impl Value {
    /// Check if the value is invalid.
    pub fn is_none(&self) -> bool {
        matches!(self, Value::None)
    }

    /// Calculate the power of two values, if possible.
    pub fn pow(&self, rhs: &Value) -> ValueResult {
        match (&self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => lhs.pow(rhs),
            _ => Err(ValueError::InvalidOperator("^".to_string())),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let ordering = match (self, other) {
            (Value::Number(lhs), Value::Number(rhs)) => lhs.partial_cmp(rhs),
            _ => None,
        };
        if ordering.is_none() {
            log::warn!("unhandled type mismatch between {self} and {other}");
        }
        ordering
    }
}

impl crate::ty::Ty for Value {
    fn ty(&self) -> Type {
        match self {
            Value::None => Type::Invalid,
            Value::Number(number) => number.ty(),
            Value::Bool(_) => Type::Bool,
            Value::String(_) => Type::String,
            Value::List(list) => list.ty(),
            Value::Tuple(tuple) => tuple.ty(),
            Value::Matrix(matrix) => matrix.ty(),
            Value::Model(model) => model.ty(),
        }
    }
}

macro_rules! impl_try_from {
    ($($variant:ident),+ => $ty:ty ) => {
        impl TryFrom<Value> for $ty {
            type Error = ValueError;

            fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    $(Value::$variant(v) => Ok(v.into()),)*
                    value => Err(ValueError::CannotConvert(value.to_string(), stringify!($ty).into())),
                }
            }
        }
    };
}

impl_try_from!(Bool => bool);
impl_try_from!(String => String);

impl TryFrom<Value> for Integer {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(Number::Integer(integer)) => Ok(integer),
            value => Err(ValueError::CannotConvert(
                value.to_string(),
                "Integer".into(),
            )),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Integer::try_from(value).map(Into::into)
    }
}

impl_try_from!(List => List);

impl TryFrom<Value> for Scalar {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(Number::Integer(i)) => Ok(Scalar::from(i)),
            Value::Number(Number::Quantity(Quantity {
                value,
                quantity_type: QuantityType::Scalar,
                ..
            })) => Ok(value),
            _ => Err(ValueError::CannotConvert(
                value.to_string(),
                "Scalar".into(),
            )),
        }
    }
}

impl TryFrom<Value> for Length {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(Number::Quantity(Quantity {
                value,
                quantity_type: QuantityType::Length,
                ..
            })) => Ok(Length(value)),
            _ => Err(ValueError::CannotConvert(
                value.to_string(),
                "Length".into(),
            )),
        }
    }
}

impl TryFrom<Value> for Rc<List> {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::List(list) => Ok(list),
            _ => Err(ValueError::CannotConvert(value.to_string(), "List".into())),
        }
    }
}

impl TryFrom<Value> for Angle {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(Number::Quantity(Quantity {
                value,
                quantity_type: QuantityType::Angle,
                ..
            })) => Ok(Angle(value)),
            _ => Err(ValueError::CannotConvert(value.to_string(), "Angle".into())),
        }
    }
}

impl TryFrom<Value> for Rc<ModelTree> {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Model(tree) => Ok(tree),
            _ => Err(ValueError::CannotConvert(
                value.to_string(),
                "ModelTree".into(),
            )),
        }
    }
}

impl TryFrom<Value> for Mat3 {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Matrix(m) = &value
            && let Matrix::Matrix3(matrix3) = m.as_ref()
        {
            Ok(*matrix3)
        } else {
            Err(ValueError::CannotConvert(value.to_string(), "Mat3".into()))
        }
    }
}

impl TryFrom<Value> for Vec2 {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Tuple(tuple) = &value {
            Ok(Vec2::new(
            	Length::try_from(tuple.get_field(&"x".into()).unwrap().clone()).unwrap().0,
             	Length::try_from(tuple.get_field(&"y".into()).unwrap().clone()).unwrap().0,
            ))
        } else {
            Err(ValueError::CannotConvert(value.to_string(), "Vec2".into()))
        }
    }
}


impl From<Quantity> for Value {
    fn from(quantity: Quantity) -> Self {
        Value::Number(Number::Quantity(quantity))
    }
}

impl From<Integer> for Value {
    fn from(integer: Integer) -> Self {
        Value::Number(Number::Integer(integer))
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Integer::from_num(value).into()
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Quantity::from(Scalar::from_num(f)).into()
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Quantity::from(Scalar::from_num(f)).into()
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Integer::from_num(i).into()
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Integer::from_num(i).into()
    }
}

impl From<Scalar> for Value {
    fn from(scalar: Scalar) -> Self {
        Quantity::from(scalar).into()
    }
}

impl From<Length> for Value {
    fn from(length: Length) -> Self {
        Quantity::from(length).into()
    }
}

impl From<Color> for Value {
    fn from(color: Color) -> Self {
        Self::Tuple(Rc::new(color.into()))
    }
}

impl From<Vec3> for Value {
    fn from(v: Vec3) -> Self {
        Self::Tuple(Rc::new(v.into()))
    }
}

impl From<List> for Value {
    fn from(list: List) -> Self {
        Self::List(Rc::new(list))
    }
}

impl From<Tuple> for Value {
    fn from(tuple: Tuple) -> Self {
        Value::Tuple(Rc::new(tuple))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s.to_compact_string())
    }
}

impl From<Mat3> for Value {
    fn from(m: Mat3) -> Self {
        Self::Matrix(Rc::new(Matrix::Matrix3(m)))
    }
}

impl From<Model> for Value {
    fn from(model: Model) -> Self {
        Self::Model(Rc::new(model.into()))
    }
}

impl From<ModelTree> for Value {
    fn from(model: ModelTree) -> Self {
        Self::Model(Rc::new(model))
    }
}

impl From<&'static str> for Value {
    fn from(s: &'static str) -> Self {
        Self::String(s.to_compact_string())
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::None
    }
}
impl FromIterator<Value> for Value {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        List::from_iter(iter).into()
    }
}
