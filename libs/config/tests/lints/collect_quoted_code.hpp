ignore1 = "abe_justText";
select1 = "call cba_fnc_someFunc"; // selected
select2 = "abe_fnc_looksLikeProjectFuncQuoted"; // selected


class CfgVehicles {
    class Car {
        class ace_actions {
            condition = "someVar"; // selected
        };
        class MFD {
            action = "dontTryToEvalThisAsSQF * speed";
        };
    };
};
class zen_context_menu_actions {
    class x {
        statement = "[_groups,true] call someThing"; // selected
    };
};
class Cfg3DEN {
    expression = "_this setVariable [""var"", _value];"; // selected
};
