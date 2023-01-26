class RscDisplayEmpty;
class GVAR(MainMenuHelper): RscDisplayEmpty {
    onLoad = QUOTE(\
        (_this select 0) call FUNC(openSettingsMenu);\
        (_this select 0) closeDisplay 0;);
};

multiline = "" \n "if ((_this select 1) in [0x1C    , 0x9C    ]) then {" \n "['cba_events_chatMessageSent', [ctrlText ((_this select 0) displayctrl 101), _this select 0]] call CBA_fnc_localEvent;" \n "};" \n "false";
