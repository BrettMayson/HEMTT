#define CHECK(X, MSG) if (X) then { systemChat format [format["Pass: %1",MSG], X]; } else { systemChat format [format["Pass: %1",MSG], X]; }

private _test = true;

CHECK(_test, "CHECK(_test) with value %1");

#define PREFIX TEST
systemChat format ["%1: %2", "PREFIX", 'PREFIX'];
