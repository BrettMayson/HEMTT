try { throw "invalid argument" } catch { hint str _exception };
123 try { if (_this != 123) throw "invalid argument" } catch { hint str _exception };

private _testVariable = "a";
try {
    if (_testVariable / 2 == 16) throw "Test Variable Error!"; // 2x div error
} catch {
    hint str _exception;
};

private _testHandler = { YYY = _testVariable };
"XXX" addPublicVariableEventHandler _testHandler; // called in event scope, so _testVariable is undef

// ToDo 2.22 spawn/continueWith
