my_fnc = { systemChat "Yes!" };
call { systemChat "Maybe?"; call my_fnc };


"['ace_infoDisplayChanged',  [_this select 0,  'Any']] call CBA_fnc_localEvent;";

private _side = [west,east,independent,civilian] select ((_display getVariable ["newSide", (_display getVariable ["oldSide", 0])]));
