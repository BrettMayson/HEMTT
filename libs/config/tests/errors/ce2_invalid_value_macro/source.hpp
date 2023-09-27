#define QUOTE(x) #x
#define PATHTO(x) \some\x
#define QPATHTO(x) QUOTE(PATHTO(x))

path = PATHTO(thing);
class Test {
    path = PATHTO(thing);
};
