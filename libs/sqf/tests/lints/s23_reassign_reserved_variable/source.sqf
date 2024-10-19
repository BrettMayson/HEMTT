// error for overwriting
call {
    _this = 123;
};

// error for overwriting
call {
    params ["_this"];
};

// error for not restoring
call {
    private _savedThis = _this;
    _this = 123;
    call _my_fnc;
};

// error for saving while still saved
call {
    private _savedThis = _this;
    _this = 123;
    private _savedThis = 1234;
    call _my_fnc;
    _this = _savedThis;
};

// no error
call {
    private _savedThis = _this;
    _this = 123;
    call _my_fnc;
    _this = _savedThis;
};
