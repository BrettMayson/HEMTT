private _testA = "5";
_testA = parseNumber _testA;
systemChat _testA; // error (var is known to be number here)

private _testB = "5";
if (x) then { _testB = parseNumber _testB; };
systemChat _testB; // ok (var may still be string here)
