// Remain one line
_myArray select { alive _x };

// Become one line
_myArray select {
    alive _x
};

// Remaine multi line
_myArray select {
    private _class = typeOf _x;
    (_class == "SoldierWB" || _class == "SoldierEB") && alive _x
};

if (alive player) then { continue };

if (isServer) exitWith { false };

if (hasInterface) exitWith { 19 };
