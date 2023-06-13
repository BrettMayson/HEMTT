params [
    ["_uniforms", [], [[]]]
];

private _delay = 0.1;

sleep 1;

ps_cam cameraEffect ["INTERNAL", "BACK"];
ps_cam camSetDir vectorDir (ps_camLocations get "uniform");
ps_cam camSetPos getPos (ps_camLocations get "uniform");
ps_cam camCommit 0;

sleep _delay;

// Preload assets
{
    model_clothing setUnitLoadout "C_Soldier_VR_F";
    model_clothing forceAddUniform _x;
    sleep _delay;
} forEach _uniforms;

sleep 3;

// Take screenshots
{
    model_clothing setUnitLoadout "C_Soldier_VR_F";
    model_clothing forceAddUniform _x;
    sleep _delay;
    screenshot format ["%1.png", _x];
    sleep _delay;
    "hemtt_comm" callExtension ["photoshoot:uniform", [_x]];
} forEach _uniforms;

endMission "END1";
