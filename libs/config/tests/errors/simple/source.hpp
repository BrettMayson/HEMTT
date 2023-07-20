#define QUOTE(x) #x
#define PATHTO(x) \some\x
#define QPATHTO(x) QUOTE(PATHTO(x))

class Test {
    data = something;
    path = PATHTO(thing);
    data = "something"
    class data {
        data = "something";
    };
    class Child {
        data = "something";
    };
    class Child {
        data = "something";
    };
};
