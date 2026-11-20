// Assignments of consecutive lines align.
foo = Circle(10mm);
a   = Rect(10);

// Only the assignments of the same depth and list align.
x  = Circle(r = 5mm);
yy = Rect(width = 1mm, height = 2mm);

// A line, which needs too much padding, starts a new table.
c = 1;
very_long_variable_name = 2;
d = 3;

// The items of similar lists align.
f(a,   bb);
f(aaa, b);

// Trailing comments align.
const SUN    = 20; /* s */
const PLANET = 22; /* p */
a;      // c
longer; // d

sketch S(
    module: Length,
    teeth: Integer,
    pressure_angle = 20°,
    backlash       = 0mm,
    clearance: Length,
) {}
