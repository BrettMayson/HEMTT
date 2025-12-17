// Modified version of BIS_fnc_exportEditorPreviews

disableSerialization;

params [
    ["_delay",1,[0]],
    ["_allVehicles",0,[0,""]],
    ["_sides",[],[[]]],
    ["_mods",[],[[]]],
    ["_patches",[],[[]]],
    ["_classes",[],[[]]]
];

_product = productVersion select 1;

if (_allVehicles isEqualType "") then {
    _allVehicles = switch (toLower _allVehicles) do {
        case "props": {-1};
        case "vehicles": {1};
        default {0};
    };
};

_sides = +_sides;
{
    if (_x isEqualType sideUnknown) then {_sides set [_foreachindex,_x call bis_fnc_sideid];};
} forEach _sides;
if (count _sides == 0) then {_sides = [0,1,2,3,4,8];};

_mods = +_mods;
{
    _mods set [_foreachindex,toLower _x];
} forEach _mods;
_allMods = count _mods == 0;

_patches = +_patches;
{
    _patches set [_foreachindex,toLower _x];
} forEach _patches;
_allPatches = count _patches == 0;

_classes = +_classes;
{
    _classes set [_foreachindex,toLower _x];
} forEach _classes;
_allClasses = count _classes == 0;

_cfgAll = configFile >> "cfgvehicles" >> "all";
_restrictedModels = ["","\A3\Weapons_f\dummyweapon.p3d","\A3\Weapons_f\laserTgt.p3d","\A3\Structures_F\Mil\Helipads\HelipadEmpty_F.p3d"];
_blacklist = [
    "WeaponHolder",
    "LaserTarget",
    "Bag_Base"
];

// Original had a check for scope = 2
_cfgVehicles = "
    getNumber (_x >> 'scope') > 0
    &&
    {
        getNumber (_x >> 'side') in _sides
        &&
        {
            _class = configName _x;
            _isAllVehicles = _class isKindOf 'allvehicles';
            (_allVehicles == 0 || (_allVehicles == 1 && _isAllVehicles) || (_allVehicles == -1 && !_isAllVehicles))
            &&
            {
                (_allMods || {(toLower _x) in _mods} count (configSourceModList _x) > 0)
                &&
                {
                    (_allPatches || {(toLower _x) in _patches} count (configSourceAddonList _x) > 0)
                    &&
                    {
                        (_allClasses || {(toLower _class) in _classes})
                        &&
                        {
                            !(getText (_x >> 'model') in _restrictedModels)
                            &&
                            {
                                inheritsFrom _x != _cfgAll
                                &&
                                {
                                    {_class isKindOf _x} count _blacklist == 0
                                }
                            }
                        }
                    }
                };
            }
        }
    }
" configClasses (configFile >> "cfgVehicles");
_cfgVehiclesCount = count _cfgVehicles;

if (_cfgVehiclesCount == 0) exitWith {["No classes found!"] call bis_fnc_error;};

_posZ = 250;
_pos = [1024, 1024, _posZ];

_cam = "camera" camCreate _pos;
_cam cameraEffect ["internal","back"];
_cam camPrepareFocus [-1,-1];
_cam camPrepareFOV 0.4;
_cam camCommitPrepared 0;
showCinemaBorder false;
setAperture 45;

_sphereColor = "#(argb,8,8,3)color(0.93,1.0,0.98,0.1)";

_sphereGround = createVehicle ["Sphere_3DEN",_pos,[],0,"none"];
_sphereNoGround = createVehicle ["SphereNoGround_3DEN",_pos,[],0,"none"];
{
    _x setPosATL _pos;
    _x setDir 0;
    _x setObjectTexture [0,_sphereColor];
    _x setObjectTexture [1,_sphereColor];
    _x hideObject true;
} forEach [_sphereGround,_sphereNoGround];

_display = [] call bis_fnc_displayMission;

_screenTop = safezoneY;
_screenBottom = safezoneY + safezoneH;
_screenLeft = safezoneX;
_screenRight = safezoneX + safezoneW;

{
    _class = configName _x;

    _camDirH = 135;
    _camDirV = 15;
    _posLocal = +_pos;
    if (_class isKindOf "HeliH") then {
        _posLocal set [2,0];
        _camDirH = 90;
        _camDirV = 75;
    };

    _object = createVehicle [_class,_posLocal,[],0,"none"];
    if (_class isKindOf "allvehicles") then {_object setDir 90;} else {_object setDir 270;};
    if (primaryWeapon _object != "") then {
        _object switchMove "amovpercmstpslowwrfldnon"
    } else {
        if (handgunWeapon _object != "") then {
            _object switchMove "amovpercmstpslowwpstdnon";
        } else {
            _object switchMove "amovpercmstpsnonwnondnon";
        };
    };
    _object setPosATL _posLocal;
    _object switchAction "default";
    _timeCapture = time + _delay;
    if (_object isKindOf "FlagCarrierCore") then {
        _object spawn {_this enableSimulation false;};
    } else {
        _object enableSimulation false;
    };

    _bbox = boundingBoxReal _object;
    _bbox1 = _bbox select 0;
    _bbox2 = _bbox select 1;
    _worldPositions = [
        _object modelToWorld [_bbox1 select 0,_bbox1 select 1,_bbox1 select 2],
        _object modelToWorld [_bbox1 select 0,_bbox1 select 1,_bbox2 select 2],
        _object modelToWorld [_bbox1 select 0,_bbox2 select 1,_bbox1 select 2],
        _object modelToWorld [_bbox1 select 0,_bbox2 select 1,_bbox2 select 2],
        _object modelToWorld [_bbox2 select 0,_bbox1 select 1,_bbox1 select 2],
        _object modelToWorld [_bbox2 select 0,_bbox1 select 1,_bbox2 select 2],
        _object modelToWorld [_bbox2 select 0,_bbox2 select 1,_bbox1 select 2],
        _object modelToWorld [_bbox2 select 0,_bbox2 select 1,_bbox2 select 2]
    ];

    _pointLowest = 0;
    _pointHighest = 0;
    {
        _xZ = (_x select 2) - _posZ;
        if (_xZ > _pointHighest) then {_pointHighest = _xZ;};
        if (_xZ < _pointLowest) then {_pointLowest = _xZ;};
    } forEach _worldPositions;
    _sphere = if (abs(_pointLowest) > abs(_pointHighest * 2/3)) then {_sphereNoGround} else {_sphereGround};
    _sphere hideObject false;
    _sphere setPos _pos;

    _camAngle = _camDirV;
    _camDis = (1.5 * ((sizeof _class) max 0.1)) min 124 max 0.2;
    _camPos = [_posLocal,_camDis,_camDirH] call bis_fnc_relpos;
    _camPos set [2,((_object modelToWorld [0,0,0]) select 2) + (tan _camAngle * _camDis)];
    _cam camPreparePos _camPos;
    _cam camPrepareTarget (_object modelToWorld [0,0,0]);
    _cam camPrepareFocus [-1,-1];
    _cam camPrepareFOV 0.7;
    _cam camCommitPrepared 0;
    _cam camPreload 0;
    waitUntil { camPreloaded _cam };
    sleep 0.01;

    if (_class isKindOf "man" && !(_class isKindOf "animal")) then {
        _cam camPrepareTarget (_object modelToWorld [0,0,1.25]);
        _cam camPrepareFOV 0.075;
        _cam camCommitPrepared 0;
    } else {
        _cam camPrepareTarget _object;

        _fovStart = if (_camDis < 0.35) then {0.4} else {0.1};
        for "_f" from _fovStart to 0.7 step 0.01 do {
            _cam camPrepareFOV _f;
            _cam camCommitPrepared 0;
            sleep 0.01;
            _onScreen = true;
            {
                _screenPos = worldToScreen _x;
                if (count _screenPos == 0) then {_screenPos = [10,10];};
                if (abs (linearConversion [_screenLeft,_screenRight,_screenPos select 0,-1,1]) > 1) exitwith {_onScreen = false;};
                if (abs (linearConversion [_screenTop,_screenBottom,_screenPos select 1,-1,1]) > 1) exitwith {_onScreen = false;};
            } forEach _worldPositions;
            if (_onScreen) exitwith {};
        };
    };

    waituntil {time > _timeCapture};
    "hemtt_comm" callExtension ["photoshoot:previews:class", [_class]];
    sleep 2;

    _object setPos [10,10,10];
    deleteVehicle _object;
    _sphere hideObject true;
} forEach _cfgVehicles;

_cam cameraEffect ["terminate","back"];
camDestroy _cam;
deleteVehicle _sphereGround;
deleteVehicle _sphereNoGround;
setAperture -1;
