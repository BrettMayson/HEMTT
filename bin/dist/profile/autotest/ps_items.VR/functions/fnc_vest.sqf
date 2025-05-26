params [
    ["_class", "", [""]]
];

if (_class == "") exitWith {};

if (isNil "ps_cam") then {
    ps_cam = "camera" camCreate [0,0,0];
};

private _pos = getPos item_position;

private _holder = createVehicle ["groundweaponholder",_pos,[],0,"none"]; 
_holder setPos _pos; 
_holder addWeaponCargo [_class, 1]; 
_holder setvectordirandup [[0.00173726,0.000167279,0.999998],[-0.995395,-0.0958456,0.00177588]];

private _campos = [_pos, 2, 90] call bis_fnc_relpos; 
_campos set [2,_pos#2 + 1.2]; 
ps_cam cameraEffect ["INTERNAL", "BACK"]; 
ps_cam camSetPos _campos; 
ps_cam camSetFov 0.4; 
ps_cam camSetTarget [_pos#0,_pos#1 -0.2,_pos#2 + 0.9];
ps_cam camCommit 0;

sleep 0.5;

screenshot format ["%1.png", _class];
sleep 0.3;
"hemtt_comm" callExtension ["photoshoot:items:weapon", [_class]];

deleteVehicle _holder;

sleep 0.1;
