class external;

class basic {
    property = "some text";
    data = 12.3;
    array[] = {1, 2, 3};
    class child: external {
        property = "child text";
        expand[] += {4, 5, 6};
        delete something;
    };
};
