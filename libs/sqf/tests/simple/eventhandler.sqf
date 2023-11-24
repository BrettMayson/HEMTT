// Fine
{ deleteVehicle _x } count allPlayers;

// Fine
["something", {
    if (alive player) then {
        allPlayers findIf { alive _x };
        {
            deleteVehicle _x;
        } forEach allPlayers;
    };
}] call CBA_fnc_addEventHandler;

// Unparseable
["something", {
    if (alive player) then {
        { deleteVehicle _x } count allPlayers;
    };
}] call CBA_fnc_addEventHandler;
