// Test duplicate case detection

// Simple duplicate
switch (_value) do {
    case 1: { "one" };
    case 2: { "two" };
    case 1: { "one again" };
};

// Multiple duplicates
switch (_x) do {
    case "a": { 1 };
    case "b": { 2 };
    case "a": { 3 };
    case "b": { 4 };
};

// Variable case (should be detected)
switch (_var) do {
    case _distBottom: { [_x, 0] };
    case _distRight: { [_worldSize, _y] };
    case _distBottom: { [_x, _worldSize] };
};

// Default cases are OK (not considered cases for duplication)
switch (_value) do {
    case 1: { "one" };
    case 2: { "two" };
    case default { "other" };
};

// Single case is OK
switch (_value) do {
    case 1: { "one" };
};

