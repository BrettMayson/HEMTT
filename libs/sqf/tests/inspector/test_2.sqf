// Mainly checking wiki syntax for correct optionals

// check inner nil
obj addWeaponItem ["weapon", ["item", nil, "muzzle"], true];
obj addWeaponItem ["weapon", ["item"], true];

 // check too many/few on variadic
format ["%1 %2 %3 %4 %5", 1, 2, 3, 4, 5];
format [""];
[] params [];

// False positives on wiki
configProperties [configFile >> "ACE_Curator"];
x selectionPosition [y, "Memory"];
ropeCreate [obj1, pos1, objNull, [0, 0, 0], dist1];
lineIntersectsSurfaces [[], [], objNull, objNull, true, 2];
uuid insert [8, ["-"]];
createTrigger["EmptyDetector", [1,2,3]];
showHUD [true,false,false,false,false,false,false,true];
createVehicle ["", [0,0,0]];
x drawRectangle [getPos player, 20,	20,	getDir player, [0,0,1,1],""];

createHashMapFromArray [["empty", {0}]];
lineIntersectsObjs [eyePos player, ATLToASL screenToWorld [0.5, 0.5]];
formatText ["%1%2%3", "line1", "<br/>", "line2"];
[] select [2];
