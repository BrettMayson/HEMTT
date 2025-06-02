class CfgFunctions {
    class test_useTag {
        tag = "test_apple";
        class someCategory1 {
            class f1 {};
        };
    };
    class test_blueberry {
        class someCategory2 {
            class f2 {};
            class f3; // defined as an external, but will still be collected
        };
    };
};
