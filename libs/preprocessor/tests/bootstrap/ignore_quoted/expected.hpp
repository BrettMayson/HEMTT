
private _test = true;

if (_test) then { systemChat format [format["Pass: %1", "CHECK(_test) with value %1"], _test]; } else { systemChat format [format["Pass: %1", "CHECK(_test) with value %1"], _test]; };

systemChat format ["%1: %2", "PREFIX", 'TEST'];

if (_test) then { systemChat format [format["Pass: %1", "CHECK(_test, ...) with value %1"], _test]; } else { systemChat format [format["Pass: %1", "CHECK(_test, ...) with value %1"], _test]; };
