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

class CfgVehicles {
    class C_Soldier_VR_F;
    class HEMTTModel: C_Soldier_VR_F {
        displayName = "HEMTT Model";
        author = "HEMTT";
        hiddenSelections[] = {"Camo_arm_left","Camo_arm_right","Camo_body","Camo_head","Camo_leg_left","Camo_leg_right"};
        hiddenSelectionsMaterials[] = {
            QPATHTOF(chroma.rvmat)
            ,QPATHTOF(chroma.rvmat)
            ,QPATHTOF(chroma.rvmat)
            ,QPATHTOF(chroma.rvmat)
            ,QPATHTOF(chroma.rvmat)
            ,QPATHTOF(chroma.rvmat)
        };
        hiddenSelectionsTextures[] = {
            "#(argb,8,8,3)color(1,0,1,1,ca)"
            ,"#(argb,8,8,3)color(1,0,1,1,ca)"
            ,"#(argb,8,8,3)color(1,0,1,1,ca)"
            ,"#(argb,8,8,3)color(1,0,1,1,ca)"
            ,"#(argb,8,8,3)color(1,0,1,1,ca)"
            ,"#(argb,8,8,3)color(1,0,1,1,ca)"
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
