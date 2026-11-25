// The line breaks of the source are spaces.
x = Circle(r = 5mm)
    .extrude(5mm);
y = a
    + b;
f(1,
    2);


// Blank lines are paragraphs, comments stay on their lines.
foo // a comment breaks the chain
    .a(t);

// A comment inside brackets breaks the list.
z = [
    // a comment
    1, 2];
t = ( // a comment
    a = 1, b = 2);
r = [ // a comment
    1..2];
