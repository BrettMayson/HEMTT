private _test1 = false;
private _test2 = true;

if (_test1 && alive player) then { };
if (_test1 && _test2) then { };
if (_test1 && {call x; _test2}) then { };
if (_test1 && {_test2}) then { };

if (!isNil "someVar" && {somevar}) then {}; // ignore
