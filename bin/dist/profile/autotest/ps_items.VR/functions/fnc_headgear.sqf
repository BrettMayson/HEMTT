params [["_class", "", [""]]];

if (_class == "") exitWith {};

if (isNil "ps_cam") then { ps_cam = "camera" camCreate [0, 0, 0]; };

private _pos = getPos item_position;

private _holder = createVehicle ["groundweaponholder", _pos, [], 0, "none"];
_holder setPos _pos;
_holder addWeaponCargo [_class, 1];
_holder setVectorDirAndUp [[0, 0, 1], [0, -1, 0]];

private _campos = [_pos, 2.5, 20] call bis_fnc_relpos;
_campos set [2, _pos#2 + 2.5];
ps_cam cameraEffect ["INTERNAL", "BACK"];
ps_cam camSetPos _campos;
ps_cam camSetFov 0.1;
ps_cam camSetTarget [_pos#0, _pos#1 + 0.425, _pos#2 + 1.3];
ps_cam camCommit 0;

sleep 0.5;

screenshot format ["%1.png", _class];
sleep 0.3;
"hemtt_comm" callExtension ["photoshoot:items:weapon", [_class]];

deleteVehicle _holder;

sleep 0.1;
