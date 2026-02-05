private _workingArms = if (alive player) then { 2 } else { 0 };
private _workingLegs = if (alive player) then { systemChat "arm"; 2 } else { 0 };
if (alivePlayer) then {
    private _thing = if (alive player) then { 2 } else { 0 };
};
private _limbs = [
    if (alive player) then { "torso" } else { "legs" },
    if (alive player) then { "torso" } else { "legs" }
];

private _isAlive = if (alive player) then { true } else { false };
private _isDry = if (surfaceIsWater position player) then { false } else { true };
private _isWarmStr = if (temperature > 20) then { "true" } else { "false" };

if (isNil "someVar") then { 5 } else { someVar };
if (someLogic && {!isNil "otherVar"}) then { otherVar } else { 6 };
