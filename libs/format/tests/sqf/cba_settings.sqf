[
    QGVAR(keepEngineRunning),
    "CHECKBOX",
    [LSTRING(SettingKeepEngineRunningName), LSTRING(SettingKeepEngineRunningDesc)],
    ELSTRING(common,ACEKeybindCategoryVehicles),
    false, // default value
    true // isGlobal
] call CBA_fnc_addSetting;
