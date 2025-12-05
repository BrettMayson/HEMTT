// Should NOT trigger: uses simple variables (could be angles, not positions)
// The lint only triggers when it sees 'select' operations to avoid false positives
private _pos1 = [100, 200];
private _pos2 = [150, 250];
private _x1 = _pos1 select 0;
private _y1 = _pos1 select 1;
private _x2 = _pos2 select 0;
private _y2 = _pos2 select 1;
private _dist2D = sqrt((_x1 - _x2)^2 + (_y1 - _y2)^2);

// Should NOT trigger: uses simple variables (could be arbitrary calculations)
private _distSqr2D = (_x1 - _x2)^2 + (_y1 - _y2)^2;

// Should NOT trigger: uses simple variables
private _pos3d1 = [100, 200, 50];
private _pos3d2 = [150, 250, 75];
private _x3 = _pos3d1 select 0;
private _y3 = _pos3d1 select 1;
private _z1 = _pos3d1 select 2;
private _x4 = _pos3d2 select 0;
private _y4 = _pos3d2 select 1;
private _z2 = _pos3d2 select 2;
private _dist3D = sqrt((_x3 - _x4)^2 + (_y3 - _y4)^2 + (_z1 - _z2)^2);

// Should NOT trigger: uses simple variables
private _distSqr3D = (_x3 - _x4)^2 + (_y3 - _y4)^2 + (_z1 - _z2)^2;

// Should trigger warning: using select inline indicates clear position access
// This pattern is unambiguous - we're extracting coordinates from position arrays
private _dist3D = sqrt(((_pos3d1 select 0) - (_pos3d2 select 0))^2 + 
                          ((_pos3d1 select 1) - (_pos3d2 select 1))^2 + 
                          ((_pos3d1 select 2) - (_pos3d2 select 2))^2);

// Should NOT trigger: not a squared difference
private _notDist1 = sqrt(_x1 + _y1);

// Should NOT trigger: using the built-in distance function
private _correct1 = _pos1 distance _pos2;
private _correct2 = _pos1 distance2D _pos2;
private _correct3 = _pos1 distanceSqr _pos2;
