private _test1 = call {
    private _temp1 = 5;
    // TRACE_1("disabled debug example",_temp1);
    _temp1; // reduced
};

private _test2 = call {
    _temp2 = 10;
    _temp2; // not local, skipped
};
