private _recoil = "str";
if (isClass (configFile >> "CfgRecoils" >> _recoil)) then {
    _recoil = getArray (configFile >> "CfgRecoils" >> _recoil >> "kickBack"); // recoil is now guaranteed to be an array
    if (isNumber (configFile >> "CfgRecoils" >> _recoil >> "xyz")) then { // error
    };
};

private _test1 = "5";
if (someCondition) then {
    _test1 = 5;
    x = _test1 + "a" // error
};
y = _test1 + "b"; // line8's _test1 could be string or number, so this is "ok"

private _test2 = "6"; // "root" starts as string
if (someConditionX) then {
    _test2 = 6; // number
    if (someConditionY) then {
        _test2 = "7"; // back to string
    };
    x = _test2 + ""; // "ok" because of conditional scopes
};
x = _test2 + 1; // "ok" because of conditional scopes
