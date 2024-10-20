params ["_a", "_b"];

params ["_a", "_b", ["_c", []]];

missionNamespace getVariable ["a", -1];

z setVariable ["b", [], true];

[1,0] vectorAdd p;

positionCameraToWorld [10000, 0, 10000];

// not consumable array
random [0, _x, 1];

private _z = if (time > 10) then { 1;2;3;4; } else { -1;-2; };

param ["_d"];

[] param ["_e"];
