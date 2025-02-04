ps_preview = [];

ps_fnc_backpack = compile preprocessFileLineNumbers "functions\fnc_backpack.sqf";
ps_fnc_firearm = compile preprocessFileLineNumbers "functions\fnc_firearm.sqf";
ps_fnc_headgear = compile preprocessFileLineNumbers "functions\fnc_headgear.sqf";
ps_fnc_uniform = compile preprocessFileLineNumbers "functions\fnc_uniform.sqf";
ps_fnc_vest = compile preprocessFileLineNumbers "functions\fnc_vest.sqf";

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
                if !(isClass (configFile >> "CfgWeapons" >> _data)) exitWith {
                    "hemtt_comm" callExtension ["photoshoot:weapon_unsupported", [_data]];
                };
                "hemtt_comm" callExtension ["log", ["debug", format ["Checking Weapon: %1", _data]]];
                private _type = getNumber (configFile >> "CfgWeapons" >> _data >> "ItemInfo" >> "type");
                "hemtt_comm" callExtension ["log", ["debug", format ["Type: %1", _type]]];
                switch (_type) do {
                    // case 1: {
                    //     // Primary
                    //     "hemtt_comm" callExtension ["log", ["debug", format ["Primary: %1", _data]]];
                    //     [_data] spawn ps_fnc_firearm;
                    // };
                    // case 2: {
                    //     // Handgun
                    //     "hemtt_comm" callExtension ["log", ["debug", format ["Handgun: %1", _data]]];
                    //     [_data, true] spawn ps_fnc_firearm;
                    // };
                    // case 3: {
                    //     // Secondary
                    //     "hemtt_comm" callExtension ["log", ["debug", format ["Secondary: %1", _data]]];
                    //     [_data] spawn ps_fnc_firearm;
                    // };
                    // case 101: {
                    //     // Muzzle
                    //     "hemtt_comm" callExtension ["log", ["debug", format ["Muzzle: %1", _data]]];
                    //     [_data] spawn ps_fnc_muzzle;
                    // };
                    // case 201: {
                    //     // Optic
                    //     "hemtt_comm" callExtension ["log", ["debug", format ["Optic: %1", _data]]];
                    //     [_data] spawn ps_fnc_optic;
                    // };
                    // case 301: {
                    //     // Flashlight
                    //     "hemtt_comm" callExtension ["log", ["debug", format ["Flashlight: %1", _data]]];
                    //     [_data] spawn ps_fnc_flashlight;
                    // };
                    // case 602: {
                    //     // NVG
                    //     "hemtt_comm" callExtension ["log", ["debug", format ["NVG: %1", _data]]];
                    //     [_data] spawn ps_fnc_nvg;
                    // };
                    case 605: {
                        // Headgear
                        "hemtt_comm" callExtension ["log", ["debug", format ["Headgear: %1", _data]]];
                        [_data] spawn ps_fnc_headgear;
                    };
                    case 701: {
                        // Vest
                        "hemtt_comm" callExtension ["log", ["debug", format ["Vest: %1", _data]]];
                        [_data] spawn ps_fnc_vest;
                    };
                    case 801: {
                        // Uniform
                        "hemtt_comm" callExtension ["log", ["debug", format ["Uniform: %1", _data]]];
                        [_data] spawn ps_fnc_uniform;
                    };
                    default {
                        // unsupported
                        "hemtt_comm" callExtension ["photoshoot:weapon_unsupported", [_data]];
                    };
                };
            };
            case "vehicle_add": {
                private _type = getText (configFile >> "CfgVehicles" >> _data >> "vehicleClass");
                "hemtt_comm" callExtension ["log", ["debug", format ["Type: %1", _type]]];
                switch (_type) do {
                    case "Backpacks": {
                        "hemtt_comm" callExtension ["log", ["debug", format ["Backpack: %1", _data]]];
                        [_data] spawn ps_fnc_backpack;
                    };
                    default {
                        "hemtt_comm" callExtension ["photoshoot:vehicle_unsupported", [_data]];
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

showCinemaBorder false;

0 spawn {
    // it fades in
    sleep 1;
    diag_log "Photoshoot: Ready";
    diag_log format ["response: %1", "hemtt_comm" callExtension ["photoshoot:ready", []]];

    if (isNil "ps_cam") then {
        ps_cam = "camera" camCreate [0,0,0];
        showCinemaBorder false;
    };
};
