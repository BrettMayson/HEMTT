private _units = units player;

// reported
private _tanks = {_x isKindOf "Tank"} count _units;
private _medics = {_x isKindOf "B_medic_F"} count allUnits;

// not the magic _x, ignore
private _other = {_y isKindOf "Tank"} count _units;

// unrelated counts, ignore
private _alive = {alive _x} count _units;
private _plain = count _units;
