private _condition = alive player && uiNamespace getVariable ["someVar", false];

private _renderDistance = [3000, 500] select (profileNamespace getVariable ["performanceMode", false]);

[] call (uiNamespace getVariable "cba_fnc_directCall"); // safe because _fnc_

[] call (uiNamespace getVariable "my_cf_code"); // ignored in option


[_condition, _renderDistance] // dummy use vars to avoid "unused variable" warnings
