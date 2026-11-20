1 + 2;
(1 + 2);
((1 + 2) + 3);
-1;
-(1 + 3);

std::geo2d::Circle(40mm);

attribute#access;
list[access];
tuple.access;
method.call(10mm);

// Two method calls are on one line.
Circle(r = 5mm).extrude(5mm);
gear.profile.hull().extrude(3mm);

builder
    .foo()
    .bar()
    .baz()
    .bof();

a_more_complex_builder
    .foo(
        a = 1mm,
        b = 2mm,
        c = 3mm,
        d = 4mm,
        e = 5mm,
    )
    .bar(a = 1, b = 2)
    .baz()
    .bof();

// The method calls behind a body are not indented.
{
    a;
}.union();

{
    a;
}
.union()
.translate(z = 1mm)
.center();
