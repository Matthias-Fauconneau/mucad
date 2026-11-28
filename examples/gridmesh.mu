use std::math::*; // {sin, pow}
use std::surface3d::GridMesh;

const LEN   = 76mm;
const WIDTH = 30mm;
const ARCH  = 4.5mm;
const CUP   = 1.2mm;
const DROOP = 9.0mm;

// Ovate-elliptic width profile (v in 0..1 from base to acute apex)
fn leaf_half_width(v: Scalar) -> Length {
    1/2 * WIDTH * pow(sin(PIrad * pow(v, 2/3)), 19/20)
}

// Gentle upward midrib arch + downward tip droop
fn midrib_z(v: Scalar) -> Length {
    ARCH * sin(PIrad * v) - DROOP * pow(v, 2)
}

// Subtle upward V-cupping of the margins relative to the midrib
fn cup_z(v: Scalar) -> Length {
    CUP * sin(PIrad * v)
}

fn leaf(v: Scalar) -> [Coord3] {
    x = LEN * v;
    w = leaf_half_width(v);
    z = midrib_z(v);
    c = cup_z(v);
    [(x= x, y= w, z= z + c), (x= x, y= 0mm, z= z), (x= x, y= -w, z= z + c)]
}

fn unit_partition(n: Integer) -> [Scalar] { [0..n]/n }

GridMesh(leaf(unit_partition(6)))