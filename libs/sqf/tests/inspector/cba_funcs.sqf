// first arg needs to be STRING
[] call cba_fnc_localEvent; // missing
[6,7] call cba_fnc_localEvent; // wrong type
6 call cba_fnc_localEvent; // wrong type
["eventName", true] call cba_fnc_localEvent; // OK
"event" call cba_fnc_localEvent; // OK
["event", [], false] call cba_fnc_localEvent; // OK (extra args)
