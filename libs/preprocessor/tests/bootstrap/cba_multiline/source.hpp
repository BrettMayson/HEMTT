class RscDisplayEmpty;
class GVAR(MainMenuHelper): RscDisplayEmpty {
    onLoad = QUOTE(\
        (_this select 0) call FUNC(openSettingsMenu);\
        (_this select 0) closeDisplay 0;);
};
