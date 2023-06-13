ps_uniforms = [];

ps_camLocations = createHashmapFromArray [
    ["uniform", camera_uniform]
];

ps_models = createHashmapFromArray [
    ["uniform", model_clothing],
    ["vest", model_clothing]
];

{
    _y hideObject true;
} forEach ps_camLocations;

{
    _y setFace "HEMTTPhotoshoot";
    _y enableSimulation false;
} forEach ps_models;

ps_cam = "camera" camCreate [0,0,0];

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_function", "_data"];
    diag_log format ["%1: %2", _name, _function];
    if (_name isEqualTo "hemtt_photoshoot") then {
        switch (_function) do {
            case "uniform": {
                diag_log format ["Uniform: %1", _data];
                ps_uniforms pushBack _data;
            };
            case "done": {
                diag_log "Starting";
                [ps_uniforms] spawn compile preprocessFileLineNumbers "functions\fnc_uniforms.sqf";;
            };
        };
    };
}];
"hemtt_comm" callExtension ["photoshoot:ready", []];
