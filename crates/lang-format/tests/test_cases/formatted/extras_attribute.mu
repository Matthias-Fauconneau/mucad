// Comment
const A = 3.0;

#[color = "red"] // Comment behind an attribute
sketch B() {
    init(d: Length) {
        c = 1;
    }

    // Comment in front of an attribute
    #[color = "blue"]
    Circle(d);
}

std::assert(1 <= 2 and 2 >= 1);
