#include "script_component.hpp"

class CfgPatches {
    class ADDON {
        name = QUOTE(COMPONENT);
        units[] = {};
        weapons[] = {};
        requiredVersion = REQUIRED_VERSION;
        requiredAddons[] = {
            "A3_Data_F_Mod_Loadorder"
        };
        VERSION_CONFIG;
    };
};

class CfgFaces {
    class Default;
    class Man_A3: Default {
        class Default {};
        class WhiteHead_01: Default {};
        class HEMTTPhotoshoot: WhiteHead_01 {
            author = "HEMTT";
            displayName = "HEMTT Photoshoot";
            texture = "#(argb,8,8,3)color(1,0,1,1,ca)";
            textureHL = "#(argb,8,8,3)color(1,0,1,1,ca)";
            textureHL2 = "#(argb,8,8,3)color(1,0,1,1,ca)";
            material = QPATHTOF(chroma.rvmat);
            materialHL = QPATHTOF(chroma.rvmat);
            materialHL2 = QPATHTOF(chroma.rvmat);
        };
    };
};

class CfgIdentities {
    class HEMTTPhotoshoot {
        face = "HEMTTPhotoshoot";
        glasses = "None";
        name = "HEMTTPhotoshoot";
        nameSound = "Kerry";
        pitch = 1.0;
        speaker = "Male01ENG";
    };
};
