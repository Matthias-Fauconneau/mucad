// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad quantity type

use mucad_lang_base::element::{UnaryOperator, BinaryOperator};
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

use crate::{TypeError, ty::Unit};

/// A quantity type with
#[derive(
    Clone, Debug, IntoStaticStr, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum QuantityType {
    /// A unitless scalar value.
    Scalar,
    /// Length in mm.
    Length,
    /// Area in mm².
    Area,
    /// Volume in mm³.
    Volume,
    /// Density in g/mm³
    Density,
    /// An angle in radians.
    Angle,
    /// Weight of a specific volume of material.
    Weight,
}

pub type QuantityTypeResult = Result<QuantityType, TypeError>;

impl QuantityType {
    /// Return base unit
    pub fn base_unit(&self) -> Unit {
        use {QuantityType::*, Unit::*};
        match self {
            Scalar => None,
            Length => Millimeter,
            Area => Millimeter2,
            Volume => Millimeter3,
            Density => GramPerMeter3,
            Angle => Rad,
            Weight => Gram,
        }
    }
    pub fn sqrt(&self) -> QuantityTypeResult {
        use QuantityType::*;
        match self {
            Area => Ok(Length),
            ty => Err(TypeError::UnsupportedUnaryOperator{op: UnaryOperator::Sqrt, ty: ty.clone().into()})
        }
    }
}

impl std::fmt::Display for QuantityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        write!(f, "{name}")
    }
}

impl std::ops::Add for QuantityType {
    type Output = QuantityTypeResult;

    fn add(self, rhs: Self) -> Self::Output {
        let lhs = self;
        Ok(match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => lhs,
            (lhs, rhs) => {
                return Err(TypeError::UnsupportedBinaryOperator {
                    op: BinaryOperator::Add,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                });
            }
        })
    }
}

impl std::ops::Sub for QuantityType {
    type Output = QuantityTypeResult;

    fn sub(self, rhs: Self) -> Self::Output {
        let lhs = self;
        Ok(match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => lhs,
            (lhs, rhs) => {
                return Err(TypeError::UnsupportedBinaryOperator {
                    op: BinaryOperator::Subtract,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                });
            }
        })
    }
}

impl std::ops::Mul for QuantityType {
    type Output = QuantityTypeResult;

    fn mul(self, rhs: Self) -> Self::Output {
        use QuantityType::*;
        let lhs = self;
        Ok(match (lhs, rhs) {
            (Length, Length) => Area,
            (ty, Scalar) | (Scalar, ty) => ty,
            (lhs, rhs) => {
                return Err(TypeError::UnsupportedBinaryOperator {
                    op: BinaryOperator::Multiply,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                });
            }
        })
    }
}

impl std::ops::Div for QuantityType {
    type Output = QuantityTypeResult;

    fn div(self, rhs: Self) -> Self::Output {
        use QuantityType::*;
        let lhs = self;
        Ok(match (lhs, rhs) {
            (ty, Scalar) => ty,
            (lhs, rhs) if lhs == rhs => Scalar,
            (lhs, rhs) => {
                return Err(TypeError::UnsupportedBinaryOperator {
                    op: BinaryOperator::Divide,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                });
            }
        })
    }
}
