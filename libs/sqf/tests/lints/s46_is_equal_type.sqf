private _a = "hello";
private _b = 5;

// both sides are typeName, reported
if (typeName _a == typeName _b) then { };
if (typeName _a isEqualTo typeName _b) then { };

// negated forms, reported as !(... isEqualType ...)
if (typeName _a != typeName _b) then { };
if (typeName _a isNotEqualTo typeName _b) then { };

// only one side is typeName, left to the static_typename lint
if (typeName _a == "STRING") then { };
if ("STRING" == typeName _a) then { };

// both sides are typeName, so this is reported here as well as by static_typename
if (typeName _a == typeName "") then { };

// not a comparison of types, ignore
if (_a isEqualType _b) then { };
if (count _a == count _b) then { };
