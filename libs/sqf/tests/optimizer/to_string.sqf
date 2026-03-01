#define GVAR(x) GEE_##x
#define QUOTE(x) #x

// b = {"double q"};
// b1 = toString b;
b2 = toString {"double q"};

// c = {'single q'};
// c1 = toString c;
c2 = toString {'single q'};

// d = { " hello 'you there' " };
// d1 = toString d;
d2 = toString { " hello 'you there' " };

// e = { ' hello "you there" ' };
// e1 = toString e;
e2 = toString { ' hello "you there" ' };

_f ctrlSetEventHandler ["ButtonClick", toString {
    x = "clicked";
}];

g = toString {123} + toString {456};
