params [
    ["_class", "", [""]]
];

if (_class == "") exitWith {};

if (isNil "ps_cam") then {
    ps_cam = "camera" camCreate [0,0,0];
};

ps_cam cameraEffect ["INTERNAL", "BACK"];
ps_cam camSetTarget objNull;
ps_cam camSetDir vectorDir camera_uniform;
ps_cam camSetPos getPos camera_uniform;
hideObject camera_uniform;
ps_cam camCommit 0;
model_clothing hideObject false;

// Take screenshot
model_clothing setUnitLoadout "C_Soldier_VR_F";
removeUniform model_clothing;
model_clothing addBackpack _class;
model_clothing setFace "HEMTTPhotoshoot";
model_clothing setDir 0;
sleep 0.3;
waitUntil { 10 preloadObject model_clothing };
screenshot format ["%1.png", _class];
sleep 0.3;
model_clothing hideObject true;
"hemtt_comm" callExtension ["photoshoot:vehicle", [_class]];
