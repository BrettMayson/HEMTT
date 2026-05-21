// warn
zA = if (a) then { 1 + 1  } else { "string" };
// warn
zB = switch (c) do {
    case 1: { 1 + 1 };
    case 2: { "string" };
};
// warn
zC = [] call {
    if (random 1 > 0.5) exitWith { player };
    []
};
// diff returns, but value is not stored or used
if (d) then { createMarker ["markername", player] } else { [] pushBack 5 };
