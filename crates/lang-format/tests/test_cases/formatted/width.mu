// A list breaks, if it does not fit behind the text in front of it.
p = [(x = 1, y = 2), (x = 3, y = 4)];
a_very_long_variable_name_in_front_of_the_list = [
    (x = 10, y = 20),
    (x = 30, y = 40),
    (x = 50, y = 60),
];

// Two method calls break, if they do not fit.
cutter = InvoluteGearProfile(module = MODULE, teeth = RING, backlash = -BACKLASH, clearance = 0mm)
    .rotate(180° / RING)
    .translate(x = 1mm);

// An operator starts an indented line, if the operation does not fit.
hole = Circle(d = 9mm).extrude(20mm).translate(z = RING_RADIUS - 2mm)
    | Circle(d = 15mm).extrude(5mm).translate(z = HOUSING_RADIUS - 3mm);

// The arguments break, if a single method call does not fit.
Ring(outer_radius = HOUSING_RADIUS, inner_radius = RING_RADIUS - 2mm).extrude(3mm);
very_long_function_name(first_argument = 1mm, second_argument = 2mm).method_with_long_name(
    third_argument = 3mm,
);
