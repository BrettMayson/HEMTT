































































































































































































































































































































class CfgPatches {
class ace_advanced_ballistics {
name = "ACE2 - Advanced Ballistics";
units[] = {};
weapons[] = {};
requiredVersion = 2.10;
requiredAddons[] = {"ace_ballistics", "ace_weather"};
author = "$STR_ace_common_ACETeam";
authors[] = {"Ruthberg"};
url = "$STR_ace_main_URL";
version = 3.15; versionStr = "3.15.2.69"; versionAr[] = {3,15,2,69};
};
};


class Extended_PreStart_EventHandlers {
class ace_advanced_ballistics {
init = "call compileScript ['\z\ace\addons\advanced_ballistics\XEH_preStart.sqf']";
};
};

class Extended_PreInit_EventHandlers {
class ace_advanced_ballistics {
init = "call compileScript ['\z\ace\addons\advanced_ballistics\XEH_preInit.sqf']";
};
};

class Extended_PostInit_EventHandlers {
class ace_advanced_ballistics {
init = "call compileScript ['\z\ace\addons\advanced_ballistics\XEH_postInit.sqf']";
};
};


class CfgVehicles {
class ACE_Module;
class ace_advanced_ballistics_ModuleSettings: ACE_Module {
scope = 1;
displayName = "$STR_ace_advanced_ballistics_DisplayName";
icon = "\z\ace\addons\advanced_ballistics\UI\Icon_Module_Wind_ca.paa";
category = "ACE";
function = "ace_advanced_ballistics_fnc_initModuleSettings";
functionPriority = 1;
isGlobal = 1;
isTriggerActivated = 0;
author = "Ruthberg";
class Arguments {
class enabled {
displayName = "$STR_ace_advanced_ballistics_enabled_DisplayName";
description = "$STR_ace_advanced_ballistics_enabled_Description";
typeName = "BOOL";
defaultValue = 0;
};
class muzzleVelocityVariationEnabled {
displayName = "$STR_ace_advanced_ballistics_muzzleVelocityVariationEnabled_DisplayName";
description = "$STR_ace_advanced_ballistics_muzzleVelocityVariationEnabled_Description";
typeName = "BOOL";
defaultValue = 1;
};
class ammoTemperatureEnabled {
displayName = "$STR_ace_advanced_ballistics_ammoTemperatureEnabled_DisplayName";
description = "$STR_ace_advanced_ballistics_ammoTemperatureEnabled_Description";
typeName = "BOOL";
defaultValue = 1;
};
class barrelLengthInfluenceEnabled {
displayName = "$STR_ace_advanced_ballistics_barrelLengthInfluenceEnabled_DisplayName";
description = "$STR_ace_advanced_ballistics_barrelLengthInfluenceEnabled_Description";
typeName = "BOOL";
defaultValue = 1;
};
class bulletTraceEnabled {
displayName = "$STR_ace_advanced_ballistics_bulletTraceEnabled_DisplayName";
description = "$STR_ace_advanced_ballistics_bulletTraceEnabled_Description";
typeName = "BOOL";
defaultValue = 1;
};
class simulationInterval {
displayName = "$STR_ace_advanced_ballistics_simulationInterval_DisplayName";
description = "$STR_ace_advanced_ballistics_simulationInterval_Description";
typeName = "NUMBER";
defaultValue = 0.05;
};
};
class ModuleDescription {
description = "$STR_ace_advanced_ballistics_Description";
};
};
};


class RscTitles {
class RscTurretDial {
idd=-1;
onLoad="with uiNameSpace do { RscTurretDial = _this select 0 };";
movingEnable=0;
duration=5;
fadeIn="false";
fadeOut="false";
class controls {
class RscTurretDial {
idc=132949;
type=0;
style=128;
font="TahomaB";
colorBackground[]={0,0,0,0.8};
colorText[]={1,1,1,1};
x="SafeZoneX + 0.0025";
y="SafeZoneY + 0.0025";
w=0.10;
h=0.05;
sizeEx=0.03;
text="";
};
};
};

class RscProtractor {
idd=-1;
onLoad="with uiNameSpace do { RscProtractor = _this select 0 };";
movingEnable=0;
duration=60;
fadeIn="false";
fadeOut="false";
class controls {
class RscProtractorBase {
idc=132950;
type=0;
style=48;
font="TahomaB";
colorBackground[]={0,0,0,0};
colorText[]={1,1,1,1};
x="SafeZoneX + 0.001";
y="SafeZoneY + 0.001";
w=0.2;
h=0.2*4/3;
size=0.034;
sizeEx=0.027;
text="";
};
class RscProtractorMarker: RscProtractorBase {
idc=132951;
};
};
};
};


class ACE_Settings {
class ace_advanced_ballistics_enabled {
movedToSQF = 1;
};
class ace_advanced_ballistics_muzzleVelocityVariationEnabled {
movedToSQF = 1;
};
class ace_advanced_ballistics_ammoTemperatureEnabled {
movedToSQF = 1;
};
class ace_advanced_ballistics_barrelLengthInfluenceEnabled {
movedToSQF = 1;
};
class ace_advanced_ballistics_bulletTraceEnabled {
movedToSQF = 1;
};
class ace_advanced_ballistics_simulationInterval {
movedToSQF = 1;
};
};



class ACE_Extensions {
class ace_advanced_ballistics {
windows = 1;
client = 1;
};
};

