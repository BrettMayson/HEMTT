class test {
    displayName = "12.7"; // ignore
    irDotSize = "0.1/4"; // reducible
    width = "0.5 * safeZoneW"; // ignore
    sizes[] = { 0, "1", "(8-7)/3"}; // 0 and "1" ignored, 3rd is reducible
    opticsZoomInit = "1 call (uiNamespace getVariable 'cba_optics_fnc_setOpticMagnificationHelper')"; // ignore
    myThing[] = {{{{{{{{{"a", "4 + 4"}}}}}}}}};
    text = "0-9"; // ignored because name
    class myMagazine {
        initSpeed = "300"; // forced because name
    };
};
