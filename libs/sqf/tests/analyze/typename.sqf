hint typeName [];
private _thing = 1;
if (typeName 0 == typeName _thing) then {
    hint "They are the same type";
} else {
    hint "They are not the same type";
};

private _aliveIsBool = typeName true == typeName alive player;
