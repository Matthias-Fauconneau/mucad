use std::math::*;
use std::ops::*;
use std::geo3d::*;
// Groups the block it is called on. As it is an operation it uses method call syntax
op group() { @input }
fn vec3(x: Length, y: Length, z: Length) -> (Length, Length, Length) { (x, y, z) }

// Parametric Forest & Tree Controls
const TRUNK_BASE_Z = 24cm;
const TRUNK_RISE = 158cm;
const TRUNK_BASE_R = 7.8cm;
const ROOT_SPREAD_BASE = 52cm;
const BRANCH_REACH_BASE = 42cm;
const CANOPY_RADIUS_BASE = 32cm;
const CANOPY_HEIGHT_BASE = 44cm;
const FOREST_SPREAD = 1;

// Standard Organic Curve Helpers
part Segment(
    p0: (x: Length, y: Length, z: Length), r0: Length,
    p1: (x: Length, y: Length, z: Length), r1: Length
) {
    a = Sphere(radius = r0).translate(x = p0.x, y = p0.y, z = p0.z);
    b = Sphere(radius = r1).translate(x = p1.x, y = p1.y, z = p1.z);
    { a; b; }.hull();
}

fn bezier_point(
    p0: (x: Length, y: Length, z: Length),
    p1: (x: Length, y: Length, z: Length),
    p2: (x: Length, y: Length, z: Length),
    t: Scalar
) -> (Length, Length, Length) {
    mt = 1 - t;
    w0 = mt * mt;
    w1 = 2 * mt * t;
    w2 = t * t;
    w0 * p0 + w1 * p1 + w2 * p2
}

part BezierLimb(
    p0: (x: Length, y: Length, z: Length),
    p1: (x: Length, y: Length, z: Length),
    p2: (x: Length, y: Length, z: Length),
    r0: Length, r1: Length
) {
    q = bezier_point(p0, p1, p2, [0..6] * (1/6));

    Segment(q[0], r0 * 1.00 + r1 * 0.00, q[1], r0 * 0.88 + r1 * 0.12);
    Segment(q[1], r0 * 0.88 + r1 * 0.12, q[2], r0 * 0.72 + r1 * 0.28);
    Segment(q[2], r0 * 0.72 + r1 * 0.28, q[3], r0 * 0.52 + r1 * 0.48);
    Segment(q[3], r0 * 0.52 + r1 * 0.48, q[4], r0 * 0.34 + r1 * 0.66);
    Segment(q[4], r0 * 0.34 + r1 * 0.66, q[5], r0 * 0.16 + r1 * 0.84);
    Segment(q[5], r0 * 0.16 + r1 * 0.84, q[6], r0 * 0.00 + r1 * 1.00);
}

// Shared Spine & Limb Geometry Functions
fn trunk_pt(
    t: Scalar,
    scale: Scalar,
    lean_x: Scalar,
    lean_y: Scalar,
    seed: Scalar
) -> (Length, Length, Length) {
    z = (TRUNK_BASE_Z + t * TRUNK_RISE) * scale;
    wx = sin(t * 210deg + seed * 53deg) * 6.5cm * scale;
    wy = cos(t * 170deg + seed * 31deg) * 5cm * scale;
    x = lean_x * (t * 95cm + t * t * 35cm) * scale + wx;
    y = lean_y * (t * 95cm + t * t * 35cm) * scale + wy;
    vec3(x, y, z)
}

fn branch_t(j: Scalar) -> Scalar {
    0.60 + j * 0.072
}

fn branch_angle(j: Scalar, seed: Scalar) -> Angle {
    mod3 = j - floor(j / 3) * 3;
    j * 60deg + (mod3 - 1) * 15deg + seed * 59deg
}

fn branch_reach(j: Scalar, scale: Scalar) -> Length {
    mod2 = j - floor(j / 2) * 2;
    mod3 = j - floor(j / 3) * 3;
    (BRANCH_REACH_BASE + mod3 * 11cm + mod2 * 8cm) * scale
}

fn branch_rise(j: Scalar, scale: Scalar) -> Length {
    mod2 = j - floor(j / 2) * 2;
    tb = branch_t(j);
    (30cm + mod2 * 10cm + (1 - tb) * 24cm) * scale
}

fn branch_tip(
    j: Scalar,
    scale: Scalar,
    lean_x: Scalar,
    lean_y: Scalar,
    seed: Scalar
) -> (Length, Length, Length) {
    tb = branch_t(j);
    phi = branch_angle(j, seed);
    reach = branch_reach(j, scale);
    rise = branch_rise(j, scale);
    b0 = trunk_pt(tb, scale, lean_x, lean_y, seed);
    b0 + vec3(
        reach * cos(phi),
        reach * sin(phi),
        rise
    )
}

fn crown_tip(
    scale: Scalar,
    lean_x: Scalar,
    lean_y: Scalar,
    seed: Scalar
) -> (Length, Length, Length) {
    top = trunk_pt(1, scale, lean_x, lean_y, seed);
    top + vec3(
        lean_x * 22cm * scale,
        lean_y * 22cm * scale,
        36cm * scale
    )
}

// ============================================================================
// Mangrove Sub-Parts (Trunk, Stilt Prop Roots, Upper Branches, Canopy)
// ============================================================================
part MangroveTrunk(
    scale: Scalar,
    lean_x: Scalar,
    lean_y: Scalar,
    seed: Scalar
) {
    p = trunk_pt([0..5]*(1/5), scale, lean_x, lean_y, seed);
    
    Segment(p[0], TRUNK_BASE_R * 1.00 * scale, p[1], TRUNK_BASE_R * 0.90 * scale);
    Segment(p[1], TRUNK_BASE_R * 0.90 * scale, p[2], TRUNK_BASE_R * 0.78 * scale);
    Segment(p[2], TRUNK_BASE_R * 0.78 * scale, p[3], TRUNK_BASE_R * 0.65 * scale);
    Segment(p[3], TRUNK_BASE_R * 0.65 * scale, p[4], TRUNK_BASE_R * 0.52 * scale);
    Segment(p[4], TRUNK_BASE_R * 0.52 * scale, p[5], TRUNK_BASE_R * 0.39 * scale);
}

part PropRoot(
    i: Scalar,
    scale: Scalar,
    lean_x: Scalar,
    lean_y: Scalar,
    seed: Scalar
) {
    mod2 = i - floor(i / 2) * 2;
    mod3 = i - floor(i / 3) * 3;
    mod4 = i - floor(i / 4) * 4;
    mod5 = i - floor(i / 5) * 5;

    t_root = 0.03 + mod3 * 0.16 + mod4 * 0.035;
    theta = i * 30deg + (mod3 - 1) * 11deg + (mod5 - 2) * 5deg + seed * 43deg;
    spread = (ROOT_SPREAD_BASE + mod3 * 22cm + mod4 * 7cm + mod2 * 5cm) * scale;

    p0 = trunk_pt(t_root, scale, lean_x, lean_y, seed);
    p1 = p0 + vec3(
        spread * (0.70 + mod2 * 0.12) * cos(theta),
        spread * (0.70 + mod2 * 0.12) * sin(theta),
        (5 - mod3 * 2.5) * 1cm * scale
    );
    p2 = vec3(
        p0.x + spread * cos(theta),
        p0.y + spread * sin(theta),
        0cm
    );

    r0 = (2.6 + mod3 * 0.45) * 1cm * scale;
    r1 = (1.2 + mod2 * 0.25) * 1cm * scale;
    BezierLimb(p0, p1, p2, r0, r1);
}

part PropRootFork(
    i: Scalar,
    fork_dir: Scalar,
    scale: Scalar,
    lean_x: Scalar,
    lean_y: Scalar,
    seed: Scalar
) {
    mod2 = i - floor(i / 2) * 2;
    mod3 = i - floor(i / 3) * 3;
    mod4 = i - floor(i / 4) * 4;
    mod5 = i - floor(i / 5) * 5;

    t_root = 0.03 + mod3 * 0.16 + mod4 * 0.035;
    theta = i * 30deg + (mod3 - 1) * 11deg + (mod5 - 2) * 5deg + seed * 43deg;
    spread = (ROOT_SPREAD_BASE + mod3 * 22cm + mod4 * 7cm + mod2 * 5cm) * scale;

    p0 = trunk_pt(t_root, scale, lean_x, lean_y, seed);
    p1 = p0 + vec3(
        spread * (0.70 + mod2 * 0.12) * cos(theta),
        spread * (0.70 + mod2 * 0.12) * sin(theta),
        (5 - mod3 * 2.5) * 1cm * scale
    );
    p2 = vec3(
        p0.x + spread * cos(theta),
        p0.y + spread * sin(theta),
        0cm
    );

    f0 = bezier_point(p0, p1, p2, 0.46);
    ftheta = theta + fork_dir * 22deg;
    fspread = spread * 0.58;
    f1 = f0 + vec3(
        fspread * 0.65 * cos(ftheta),
        fspread * 0.65 * sin(ftheta),
        - 4cm * scale
    );
    f2 = vec3(
        f0.x + fspread * cos(ftheta),
        f0.y + fspread * sin(ftheta),
        0cm
    );

    BezierLimb(f0, f1, f2, 1.8cm * scale, 1cm * scale);
}

part MangroveRoots(
    scale: Scalar,
    lean_x: Scalar,
    lean_y: Scalar,
    seed: Scalar
) {
    // 12 primary arching stilt prop roots of varied heights and radial spreads
    PropRoot([0..11], scale, lean_x, lean_y, seed);
    
    // Secondary forked prop roots for intertwined coastal root structure
    PropRootFork(1, 1, scale, lean_x, lean_y, seed);
    PropRootFork(5, -1, scale, lean_x, lean_y, seed);
    PropRootFork(9, 1, scale, lean_x, lean_y, seed);
}

part UpperBranch(
    j: Scalar,
    scale: Scalar,
    lean_x: Scalar,
    lean_y: Scalar,
    seed: Scalar
) {
    tb = branch_t(j);
    phi = branch_angle(j, seed);
    reach = branch_reach(j, scale);
    rise = branch_rise(j, scale);

    b0 = trunk_pt(tb, scale, lean_x, lean_y, seed);
    b1 = b0 + vec3(
        reach * 0.46 * cos(phi),
        reach * 0.46 * sin(phi),
        rise * 0.28
    );
    b2 = branch_tip(j, scale, lean_x, lean_y, seed);

    rb0 = (2.5 + (1 - tb) * 1.4) * 1cm * scale;
    rb1 = 1cm * scale;
    BezierLimb(b0, b1, b2, rb0, rb1);
}

part MangroveBranches(
    scale: Scalar,
    lean_x: Scalar,
    lean_y: Scalar,
    seed: Scalar
) {
    UpperBranch([0..5], scale, lean_x, lean_y, seed);
    
    // Apical curved crown leader
    c0 = trunk_pt(0.98, scale, lean_x, lean_y, seed);
    c1 = c0 + vec3(
        lean_x * 6cm * scale,
        lean_y * 6cm * scale,
        20cm * scale
    );
    c2 = crown_tip(scale, lean_x, lean_y, seed);
    BezierLimb(c0, c1, c2, 3cm * scale, 1.1cm * scale);
}

part CanopyConeCluster(
    tip: (x: Length, y: Length, z: Length),
    j: Scalar,
    scale: Scalar,
    seed: Scalar
) {
    mod2 = j - floor(j / 2) * 2;
    mod3 = j - floor(j / 3) * 3;
    phi = branch_angle(j, seed);

    r_low = (CANOPY_RADIUS_BASE + mod3 * 5cm) * scale;
    h_low = (CANOPY_HEIGHT_BASE + mod2 * 8cm) * scale;
    r_up = r_low * 0.76;
    h_up = h_low * 0.88;

    p_low = tip + vec3(0cm, 0cm, - 8cm * scale);
    p_up = tip + vec3(
        3cm * scale * cos(phi + 40deg),
        3cm * scale * sin(phi + 40deg),
        h_low * 0.36
    );

    #[color = "#245E3B"]
    Cylinder(height = h_low, radius_top = 0cm, radius_bottom = r_low)
        .translate(x = p_low.x, y = p_low.y, z = p_low.z);

    #[color = "#347D4B"]
    Cylinder(height = h_up, radius_top = 0cm, radius_bottom = r_up)
        .translate(x = p_up.x, y = p_up.y, z = p_up.z);
}

part MangroveCanopy(scale: Scalar, lean_x: Scalar, lean_y: Scalar, seed: Scalar) {
    CanopyConeCluster(branch_tip(0, scale, lean_x, lean_y, seed), 0, scale, seed);
    CanopyConeCluster(branch_tip(1, scale, lean_x, lean_y, seed), 1, scale, seed);
    CanopyConeCluster(branch_tip(2, scale, lean_x, lean_y, seed), 2, scale, seed);
    CanopyConeCluster(branch_tip(3, scale, lean_x, lean_y, seed), 3, scale, seed);
    CanopyConeCluster(branch_tip(4, scale, lean_x, lean_y, seed), 4, scale, seed);
    CanopyConeCluster(branch_tip(5, scale, lean_x, lean_y, seed), 5, scale, seed);

    // Central crown canopy cones
    ct = crown_tip(scale, lean_x, lean_y, seed);
    c_low = ct + vec3(0cm, 0cm, - 10cm * scale);
    c_up = ct + vec3(lean_x * 4cm * scale, lean_y * 4cm * scale, 18cm * scale);

    #[color = "#245E3B"]
    Cylinder(height = 56cm * scale, radius_top = 0cm, radius_bottom = 42cm * scale)
        .translate(x = c_low.x, y = c_low.y, z = c_low.z);

    #[color = "#347D4B"]
    Cylinder(height = 48cm * scale, radius_top = 0cm, radius_bottom = 31cm * scale)
        .translate(x = c_up.x, y = c_up.y, z = c_up.z);
}

// Complete Mangrove Tree & Coastal Mangrove Forest Cluster
part MangroveTree(scale: Scalar, lean_x: Scalar, lean_y: Scalar, yaw: Angle, seed: Scalar) {
    {
        #[color = "#4B321E"]
        {
            MangroveTrunk(scale, lean_x, lean_y, seed);
            MangroveRoots(scale, lean_x, lean_y, seed);
            MangroveBranches(scale, lean_x, lean_y, seed);
        }.group();

        MangroveCanopy(scale, lean_x, lean_y, seed);
    }.group().rotate(z = yaw);
}

part MangroveForestCluster(spread: Scalar) {
    MangroveTree(1.28, 0.24, -0.10, 15deg, 1)
        .translate(x = 0m * spread, y = 0m * spread, z = 0cm);

    /*MangroveTree(1.15, 0.30, 0.14, 78deg, 2.3)
        .translate(x = 2.1m * spread, y = 1.3m * spread, z = 0cm);

    MangroveTree(1.02, -0.18, 0.22, 145deg, 3.7)
        .translate(x = -1.9m * spread, y = 1.5m * spread, z = 0cm);

    MangroveTree(0.86, -0.26, -0.16, 215deg, 4.9)
        .translate(x = -2.2m * spread, y = -1.4m * spread, z = 0cm);

    MangroveTree(0.72, 0.20, -0.24, 290deg, 6.1)
        .translate(x = 1.8m * spread, y = -1.8m * spread, z = 0cm);

    MangroveTree(1.10, 0.12, 0.25, 42deg, 7.4)
        .translate(x = 0.3m * spread, y = 2.8m * spread, z = 0cm);

    MangroveTree(0.66, -0.15, -0.20, 325deg, 8.8)
        .translate(x = -0.4m * spread, y = -2.5m * spread, z = 0cm);*/
}

MangroveForestCluster(FOREST_SPREAD);
