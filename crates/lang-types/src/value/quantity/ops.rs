// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Quantity binary operators module.

use crate::{Integer, Quantity, ValueResult};

impl std::ops::Neg for Quantity {
    type Output = Quantity;

    fn neg(self) -> Self::Output {
        self.map(|v| -v)
    }
}

impl std::ops::Add for Quantity {
    type Output = ValueResult;

    fn add(self, rhs: Self) -> Self::Output {
        let lhs = self;
        Ok(Quantity::new(lhs.value + rhs.value, (lhs.quantity_type + rhs.quantity_type)?)
                .with_unit(lhs.unit) // FIXME: silently wrong results on mismatched units
                .into())
    }
}

impl std::ops::Add<Integer> for Quantity {
    type Output = ValueResult;

    fn add(self, rhs: Integer) -> Self::Output {
        Quantity::from(rhs) + self
    }
}

impl std::ops::Add<Quantity> for Integer {
    type Output = ValueResult;

    fn add(self, rhs: Quantity) -> Self::Output {
        Quantity::from(self) + rhs
    }
}

impl std::ops::Sub for Quantity {
    type Output = ValueResult;

    fn sub(self, rhs: Self) -> Self::Output {
        let lhs = self;
        Ok(Quantity::new(lhs.value - rhs.value, (lhs.quantity_type - rhs.quantity_type)?)
        	.with_unit(lhs.unit) // FIXME: silently wrong results on mismatched units
         	.into())
    }
}

impl std::ops::Sub<Integer> for Quantity {
    type Output = ValueResult;

    fn sub(self, rhs: Integer) -> Self::Output {
        self - Quantity::from(rhs)
    }
}

impl std::ops::Sub<Quantity> for Integer {
    type Output = ValueResult;

    fn sub(self, rhs: Quantity) -> Self::Output {
        Quantity::from(self) - rhs
    }
}

impl std::ops::Mul for Quantity {
    type Output = ValueResult;

    fn mul(self, rhs: Self) -> Self::Output {
        let lhs = self;
        Ok(Quantity::new(lhs.value * rhs.value, (lhs.quantity_type * rhs.quantity_type)?).into())
    }
}

impl std::ops::Mul<Integer> for Quantity {
    type Output = ValueResult;

    fn mul(self, rhs: Integer) -> Self::Output {
        self * Quantity::from(rhs)
    }
}

impl std::ops::Mul<Quantity> for Integer {
    type Output = ValueResult;

    fn mul(self, rhs: Quantity) -> Self::Output {
        Quantity::from(self) * rhs
    }
}

impl std::ops::Div for Quantity {
    type Output = ValueResult;

    fn div(self, rhs: Self) -> Self::Output {
        let lhs = self;
        Ok(Quantity::new(lhs.value / rhs.value, (lhs.quantity_type / rhs.quantity_type)?).into())
    }
}

impl std::ops::Div<Integer> for Quantity {
    type Output = ValueResult;

    fn div(self, rhs: Integer) -> Self::Output {
        self / Quantity::from(rhs)
    }
}

impl std::ops::Div<Quantity> for Integer {
    type Output = ValueResult;

    fn div(self, rhs: Quantity) -> Self::Output {
        Quantity::from(self) / rhs
    }
}
