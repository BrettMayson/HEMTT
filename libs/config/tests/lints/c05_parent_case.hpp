class imported;
class local: Imported {
    value = 1;
};

class B {};
class C {};

class A {
    class b: B {};

    // Will be ignored, because the parent case can match the child case
    class c: c {};
};
