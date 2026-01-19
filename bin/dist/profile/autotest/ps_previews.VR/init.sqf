ps_preview = [];

ps_fnc_export = compile preprocessFileLineNumbers "fnc_export.sqf";

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_function", "_data"];
    diag_log format ["%1: %2", _name, _function];
    if (_name isEqualTo "hemtt_ps_previews" || _name isEqualTo "hemtt_ps") then {
        switch (_function) do {
            case "add": {
                diag_log format ["Preview: %1", _data];
                ps_preview pushBack _data;
            };
            case "run": {
                diag_log "Preview: Run";
                0 spawn {
                    sleep 1;
                    diag_log "Preview: Start";
                    diag_log format ["Preview: %1", count ps_preview];
                    [nil, "all", [], [], [], ps_preview] call ps_fnc_export;
                    sleep 2;
                    diag_log "Preview: Done";
                    "hemtt_comm" callExtension ["photoshoot:previews:done", []];
                };
            };
            case "done": {
                endMission "END1";
            };
            default {
                diag_log format ["Unknown: %1", _function];
                "hemtt_comm" callExtension ["log", ["error", format ["Unknown: %1", _function]]];
            };
        };
    };
}];

setWind [10, 10, true];
setDate [2035,5,28,10,0];

0 spawn {
    sleep 1;
    diag_log "Photoshoot: Previews Ready";
    diag_log format ["response: %1", "hemtt_comm" callExtension ["photoshoot:previews:ready", []]];
};
