#include "advanced_ballistics\script_component.hpp"

class CfgPatches {
    class ADDON {
        name = COMPONENT_NAME;
        units[] = {};
        weapons[] = {};
        requiredVersion = REQUIRED_VERSION;
        requiredAddons[] = {"ace_ballistics", "ace_weather"};
        author = ECSTRING(common,ACETeam);
        authors[] = {"Ruthberg"};
        url = ECSTRING(main,URL);
        VERSION_CONFIG;
    };
};

#include "advanced_ballistics\CfgEventHandlers.hpp"
#include "advanced_ballistics\CfgVehicles.hpp"
#include "advanced_ballistics\RscTitles.hpp"
#include "advanced_ballistics\ACE_Settings.hpp"

class ACE_Extensions {
    class ace_advanced_ballistics {
        windows = 1;
        client = 1;
    };
};
