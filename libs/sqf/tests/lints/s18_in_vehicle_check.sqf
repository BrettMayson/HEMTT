if (vehicle player != player) then {
    hint "You are in a vehicle!";
};

if (alive _unit && {_unit == vehicle _unit}) then {
    hint "You're alive and not in a vehicle!";
};
