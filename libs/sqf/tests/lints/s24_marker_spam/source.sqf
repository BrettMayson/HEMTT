// Message on all 4
call {
    private _marker = "m1";
    _marker setMarkerShape "ICON";
    _marker setMarkerType "hd_dot";
    _marker setMarkerColor "ColorRed";
    _marker setMarkerSize [1, 1];
};

// Message on all 3
call {
    "m1" setMarkerShape "ICON";
    "m1" setMarkerType "hd_dot";
    "m1" setMarkerSize [1, 1];
};

// 2 sets of 2
call {
    private _marker = "m1";
    _marker setMarkerShape "ICON";
    _marker setMarkerType "hd_dot";
    private _marker = "m2";
    _marker setMarkerColor "ColorRed";
    _marker setMarkerSize [1, 1];
};

// no message
call {
    private _marker = "m1";
    _marker setMarkerShapeLocal "ICON";
    _marker setMarkerTypeLocal "hd_dot";
    _marker setMarkerColorLocal "ColorRed";
    _marker setMarkerSize [1, 1];
};


// no message
call {
    "m1" setMarkerShape "ICON";
    "m2" setMarkerType "hd_dot";
};
