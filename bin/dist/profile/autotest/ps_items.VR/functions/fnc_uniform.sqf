params [
    ["_uniform", "", [""]]
];

if (_uniform == "") exitWith {};

if (isNil "ps_cam") then {
    ps_cam = "camera" camCreate [0,0,0];
};

ps_cam cameraEffect ["INTERNAL", "BACK"];
ps_cam camSetTarget objNull;
ps_cam camSetFOV 0.75;
ps_cam camCommit 0;
ps_cam camSetDir vectorDir camera_uniform;
ps_cam camSetPos getPos camera_uniform;
hideObject camera_uniform;
ps_cam camCommit 0;
ps_cam camPreload 0;
waitUntil { camPreloaded ps_cam };
model_clothing hideObject false;

// Take screenshot
model_clothing setUnitLoadout "C_Soldier_VR_F";
model_clothing forceAddUniform _uniform;
model_clothing setFace "HEMTTPhotoshoot";
model_clothing setDir 180;
waitUntil { 10 preloadObject model_clothing };

screenshot format ["%1.png", _uniform];
"hemtt_comm" callExtension ["photoshoot:items:weapon", [_uniform]];
sleep 0.3;
model_clothing hideObject true;
