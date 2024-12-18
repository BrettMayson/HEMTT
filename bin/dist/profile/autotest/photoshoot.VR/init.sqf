ps_preview = [];

ps_fnc_uniform = compile preprocessFileLineNumbers "functions\fnc_uniforms.sqf";

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_function", "_data"];
    diag_log format ["%1: %2", _name, _function];
    if (_name isEqualTo "hemtt_photoshoot") then {
        switch (_function) do {
            case "preview_add": {
                diag_log format ["Preview: %1", _data];
                ps_preview pushBack _data;
            };
            case "preview_run": {
                diag_log "Preview: Run";
                0 spawn {
                    diag_log "Preview: Start";
                    diag_log format ["Preview: %1", count ps_preview];
                    [nil, "all", [], [], [], ps_preview] call BIS_fnc_exportEditorPreviews;
                    sleep 2;
                    diag_log "Preview: Done";
                    "hemtt_comm" callExtension ["photoshoot:previews", []];
                };
            };
            case "weapon_add": {
                diag_log format ["Weapon: %1", _data];
                "hemtt_comm" callExtension ["log", ["debug", format ["Checking Weapon: %1", _data]]];
                private _allowedSlots = getArray (configFile >> "CfgWeapons" >> _data >> "allowedSlots") select 0;
                "hemtt_comm" callExtension ["log", ["debug", format ["Allowed Slots: %1", _allowedSlots]]];
                switch (_allowedSlots) do {
                    case 901: {
                        // Uniform
                        "hemtt_comm" callExtension ["log", ["debug", format ["Uniform: %1", _data]]];
                        [_data] spawn ps_fnc_uniform;
                    };
                    default {
                        // unsupported
                        "hemtt_comm" callExtension ["log", ["warn", format ["Unsupported: %1", _data]]];
                        diag_log format ["Unsupported: %1", _data];
                        "hemtt_comm" callExtension ["photoshoot:weapon_unsupported", [_data]];
                    };
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

0 spawn {
    // it fades in
    sleep 1;
    diag_log "Photoshoot: Ready";
    diag_log format ["response: %1", "hemtt_comm" callExtension ["photoshoot:ready", []]];
};
