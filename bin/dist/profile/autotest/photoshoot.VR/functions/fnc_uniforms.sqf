params [
    ["_uniform", "", [""]]
];

if (_uniform == "") exitWith {};

if (isNil "ps_cam") then {
    ps_cam = "camera" camCreate [0,0,0];
};
ps_cam cameraEffect ["INTERNAL", "BACK"];
ps_cam camSetDir vectorDir camera_uniform;
ps_cam camSetPos getPos camera_uniform;
hideObject camera_uniform;
ps_cam camCommit 0;

sleep 0.1;

// Take screenshot
model_clothing setUnitLoadout "C_Soldier_VR_F";
model_clothing forceAddUniform _uniform;
model_clothing setFace "HEMTTPhotoshoot";
waitUntil { 10 preloadObject model_clothing };
screenshot format ["%1.png", _uniform];
sleep 0.3;
"hemtt_comm" callExtension ["photoshoot:weapon", [_uniform]];
