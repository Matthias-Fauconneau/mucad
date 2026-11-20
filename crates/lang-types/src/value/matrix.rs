// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Matrix value type

use crate::{MatrixType, Scalar, Ty, Type, Vec2, Vec3, Vec4};

use serde::{Deserialize, Serialize};

/// 2D matrix type of its columns.
///
/// It only holds values, the math converts it to the floating point matrix of glam, see
/// [`crate::math::float`].
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mat2 {
    pub x: Vec2,
    pub y: Vec2,
}

/// 3D matrix type of its columns.
///
/// It only holds values, the math converts it to the floating point matrix of glam, see
/// [`crate::math::float`].
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mat3 {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

/// 4D matrix type of its columns.
///
/// It only holds values, the math converts it to the floating point matrix of glam, see
/// [`crate::math::float`].
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mat4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
}

impl Mat2 {
    /// A matrix of its columns.
    pub const fn from_cols(x: Vec2, y: Vec2) -> Self {
        Self { x, y }
    }

    /// The arrays of the columns.
    pub const fn to_cols_array_2d(self) -> [[Scalar; 2]; 2] {
        [self.x.to_array(), self.y.to_array()]
    }
}

impl Mat3 {
    /// A matrix of its columns.
    pub const fn from_cols(x: Vec3, y: Vec3, z: Vec3) -> Self {
        Self { x, y, z }
    }

    /// The arrays of the columns.
    pub const fn to_cols_array_2d(self) -> [[Scalar; 3]; 3] {
        [self.x.to_array(), self.y.to_array(), self.z.to_array()]
    }
}

impl Mat4 {
    /// A matrix of its columns.
    pub const fn from_cols(x: Vec4, y: Vec4, z: Vec4, w: Vec4) -> Self {
        Self { x, y, z, w }
    }

    /// The arrays of the columns.
    pub const fn to_cols_array_2d(self) -> [[Scalar; 4]; 4] {
        [
            self.x.to_array(),
            self.y.to_array(),
            self.z.to_array(),
            self.w.to_array(),
        ]
    }
}

/// The conversions of a matrix from and to the arrays of its columns, and its `Debug` format,
/// e.g. `Matrix2 [[1, 0], [0, 1]]`.
macro_rules! impl_matrix {
    ($matrix:ident, $n:literal, $name:literal, [$($column:ident),+]) => {
        impl From<[[Scalar; $n]; $n]> for $matrix {
            fn from([$($column),+]: [[Scalar; $n]; $n]) -> Self {
                Self { $($column: $column.into()),+ }
            }
        }

        impl From<$matrix> for [[Scalar; $n]; $n] {
            fn from(matrix: $matrix) -> Self {
                matrix.to_cols_array_2d()
            }
        }

        impl std::fmt::Debug for $matrix {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, concat!($name, " "))?;
                std::fmt::Debug::fmt(&self.to_cols_array_2d(), f)
            }
        }
    };
}

impl_matrix!(Mat2, 2, "Matrix2", [x, y]);
impl_matrix!(Mat3, 3, "Matrix3", [x, y, z]);
impl_matrix!(Mat4, 4, "Matrix4", [x, y, z, w]);

/// Matrix type
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub enum Matrix {
    /// 2x2 matrix.
    Matrix2(Mat2),
    /// 3x3 matrix.
    Matrix3(Mat3),
    /// 4x4 matrix.
    Matrix4(Mat4),
}

impl Ty for Matrix {
    fn ty(&self) -> Type {
        match self {
            Matrix::Matrix2(_) => Type::Matrix(MatrixType::new(2, 2)),
            Matrix::Matrix3(_) => Type::Matrix(MatrixType::new(3, 3)),
            Matrix::Matrix4(_) => Type::Matrix(MatrixType::new(4, 4)),
        }
    }
}

impl std::fmt::Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Matrix::Matrix2(matrix2) => write!(f, "{matrix2:?}"),
            Matrix::Matrix3(matrix3) => write!(f, "{matrix3:?}"),
            Matrix::Matrix4(matrix4) => write!(f, "{matrix4:?}"),
        }
    }
}
