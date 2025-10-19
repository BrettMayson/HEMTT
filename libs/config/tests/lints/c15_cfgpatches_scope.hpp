class CfgPatches {
    class myMod {
        units[] = {"abe_car3", "abe_car4", "abe_bar22"}; // bar sadly does not exist
        weapons[] = {"abe_gun2"};
        requiredVersion = 0.1;
        requiredAddons[] = {};
    };
};

class cfgvehicles {
    class Car;
    class abe_car1: Car { };
    class abe_car2: Car { // Missing
        scope = 2;
    };
    class abe_car3: abe_car1 {
        scope = 2;
    };
    class abe_car4: abe_car2 {
        scope = 1;
    };
    class abe_car5: abe_car2 { }; // Missing
};
class CfgWeapons {
    class Rifle;
    class abe_gun1: Rifle { };
    class abe_gun2: Rifle {
        scope = 2;
    };
    class abe_gun3: abe_gun2 {}; // Missing
    class test_gun1: Rifle {
        scope = 2;
    };
};
