ps_preview = [];

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
            case "done": {
                endMission "END1";
            };
        };
    };
}];

diag_log "Photoshoot: Ready";
diag_log format ["response: %1", "hemtt_comm" callExtension ["photoshoot:ready", []]];
