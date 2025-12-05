(teamMember player) addResources [];
(teamMember player) addResources ["A", "B", "C"];

(teamMember player) createTask [["x"], 10, "name1", "value1"];
(teamMember player) createTask [["x"], 10, "name1", "value1", "name2", "value2"];

xCtrl ctRemoveRows [];
xCtrl ctRemoveRows [0, 1, 2];

format [""];
format ["%1", "test"];

formatText [""];
formatText ["%1", "test"];

getGraphValues [
	[0, 10, 0, 100, 11, 0]
];
getGraphValues [
	[0, 10, 0, 100, 11, 0],
	0, 5,  1, 10,  2, 100
];

[player] inAreaArray [];
[player] inAreaArray [[0,0,0], [10,0,0], [10,10,0]];

createHashMap insert [];
createHashMap insert [["key", [1,2,3]], ["key", [4]]];
createHashMap insert [["key", [1,2,3]]];

createHashMapFromArray player; // will correctly error
createHashMapFromArray [];
createHashMapFromArray [["a", 1], ["b", 2], ["c", 3]];

lineIntersects [eyePos player, aimPos chopper, player, chopper];
lineIntersects [[eyePos player, aimPos chopper, player, chopper]];
lineIntersects [[eyePos player, aimPos chopper, player, chopper], [eyePos player, aimPos chopper, player, chopper]];

// params is handled in commands.rs
params ["_var1", objNull];
params [["_var2", "", ""]];
params [["_var3", objNull, [false]]];

ppEffectCreate ["ColorCorrections", 1];
ppEffectCreate [["ColorCorrections", 1]];
ppEffectCreate [["ColorCorrections", 1], ["ColorCorrections", 2], ["ColorCorrections", 3]];

set3DENAttributes [];
set3DENAttributes [[[], "ControlMP", true]];
set3DENAttributes [[get3DENSelected "Object", "ControlMP", true], [get3DENSelected "Object", "ControlMP", true]];

set3DENMissionAttributes [];
set3DENMissionAttributes [["Multiplayer", "respawn", 3]];
set3DENMissionAttributes [["Multiplayer", "respawn", 3], ["Multiplayer", "respawnDelay", 10]];


private _txt = text "Red text, right align";
_txt setAttributes ["color", "#FF0000"];
hint composeText [_txt];

(group player) setGroupId ["test"];
(group player) setGroupId ["%GroupNames :=: %GroupColors", "Alpha", "GroupColor2"];

(group player) setGroupIdGlobal ["test"];
(group player) setGroupIdGlobal ["%GroupNames :=: %GroupColors", "Alpha", "GroupColor2"];

"rendersurface" setPiPEffect [0];
"rendertarget0" setPiPEffect [3, 1, 1.0, 1.0, 0.0, [0.0, 1.0, 0.0, 0.25], [1.0, 0.0, 1.0, 1.0],  [0.199, 0.587, 0.114, 0.0]];

/*`textLogFormat` is marked as a broken command on the wiki*/
