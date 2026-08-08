private _units = units player;

// countType, reported
private _tanks = {_x isKindOf "Tank"} count _units;
private _medics = {_x isKindOf "B_medic_F"} count allUnits;

// countSide, reported, either operand order
private _west = {side _x == west} count _units;
private _east = {east == side _x} count _units;
private _civ = {side _x isEqualTo civilian} count _units;

// side of the group is not the side of the unit, ignore
private _grouped = {side group _x == west} count _units;
private _leader = {side leader _x == west} count _units;

// not the magic _x, ignore
private _other = {side _y == west} count _units;
private _kind = {_y isKindOf "Tank"} count _units;

// unrelated counts, ignore
private _alive = {alive _x} count _units;
private _plain = count _units;
