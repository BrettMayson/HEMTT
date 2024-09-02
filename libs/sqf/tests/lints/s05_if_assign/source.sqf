private _workingArms = if (alive player) then { 2 } else { 0 };
private _workingLegs = if (alive player) then { systemChat "arm"; 2 } else { 0 };
if (alivePlayer) then {
    private _thing = if (alive player) then { 2 } else { 0 };
};
private _limbs = [
    if (alive player) then { "torso" } else { "legs" },
    if (alive player) then { "torso" } else { "legs" },
];
