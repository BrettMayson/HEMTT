private _name = getText(configFile >> "CfgVehicles" >> typeOf _vehicle >> "displayName");
private _desc = getText(configFile / "CfgVehicles" / typeOf _vehicle / "descriptionShort");

player addEventHandler ["Fired", {
    x = configFile >> "CfgAmmo" >> (typeOf (_this # 6));
}];
