class thing1 {
    class alpha;
    class beta; // Not Used
};
class thing2 { // None of this class is used (uses the CfgAmmo scope one)
    class gamma; 
};
class thing3 {
    class delta;
};

class CfgAmmo {
    class thing2 {
        class gamma;
    };

    class BulletBase;
    class Default; // Not Used
    class myAmmmo: BulletBase {
        class thing1: thing1 {
            class alpha: alpha { cool = 1; };
        };
        class thing2: thing2 {
            class gamma: gamma { cool = 2; };
        };
        class thing3: thing3 {
            class delta: delta { cool = 3; };
        };
    };

};
