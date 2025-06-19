class test;
class first: test {
    value = 1;
};
class second: first {
    value = 2;
};
class third: other {
    value = 3;
};

class CfgVehicles {    
    class Logic;
    class Module_F: Logic {
        class AttributesBase {
            class Combo;
        };
    };
    class test: Module_F {
        scope = 2;
        displayName = "test";
        class Attributes { // should inherit from AttributesBase
            class Side: Combo {
                displayName = "example";
            };
        };
        class Attributes2: AttributesBase { // correct example
            class Side: Combo {
                displayName = "example";
            };
        };
    };
};
