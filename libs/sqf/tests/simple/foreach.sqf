// Fine
{
    deleteVehicle _x;
} forEach allUnits;

// Doesn't like code blocks with code blocks inside
{
    systemChat format ["%1", _x];
    {
        _x setDamage 1;
    } forEach crew _x;
} forEach allUnits;
