//use std::geo2d::PolyBezier;
//use std::geo3d::*;
//use std::ops::*;
use std::spline::Spline; // FIXME: use std::spline::Spline; fails

// 1. Closed smooth loop (4 points -> rounded pillow shape, extruded to verify watertightness)
sketch SmoothSquare() {
    std::spline::Spline(
        points = [(0mm, 0mm), (10mm, 0mm), (10mm, 10mm), (0mm, 10mm)],
        closed = true,
        tension = 0.0
    );
}

// 2. Tension = 1.0 should match a straight-edged 10x10 square
sketch TenseSquare() {
    Spline(
        points = [(0mm, 0mm), (10mm, 0mm), (10mm, 10mm), (0mm, 10mm)],
        closed = true,
        tension = 1.0
    );
}

// 3. Duplicate waypoint forces a sharp C0 corner at (10mm, 10mm) while staying smooth elsewhere
sketch TeardropWithCorner() {
    Spline(
        points = [
            (0mm, 0mm),
            (10mm, 10mm),
            (10mm, 10mm), // repeated point -> sharp tip
            (0mm, 20mm),
            (-10mm, 10mm)
        ],
        closed = true
    );
}

// 4. Open spline stroked into a ribbon/wire (tests reflected open endpoints)
sketch OpenWave() {
    Spline(
        points = [(0mm, 0mm), (10mm, 8mm), (20mm, -8mm), (30mm, 0mm)],
        closed = false
    )
    ;//.stroke(width = 1.5mm);
}

// Lay them out side-by-side and extrude to verify valid closed 2D regions
SmoothSquare().extrude(2mm);
//TenseSquare().extrude(2mm).translate(x = 18mm);
//TeardropWithCorner().extrude(2mm).translate(x = 36mm); 
//OpenWave().extrude(2mm).translate(y = -18mm);