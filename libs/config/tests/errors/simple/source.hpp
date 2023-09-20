#define QUOTE(x) #x
#define PATHTO(x) \some\x
#define QPATHTO(x) QUOTE(PATHTO(x))

class Test {
    outer = something;
    path = PATHTO(thing);
    outer = "nosemi"
    class outer {
        inner = "something";
    };
    class Child {
        inner = "something";
    };
    class Child {
        inner = "something";
    };
};
