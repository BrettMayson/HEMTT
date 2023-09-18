









    













    
    





    






















































                    
























        
    


















    

















        

    
    


    













    












                
        





















































































        




   










            






            






                
class CfgPatches {
    class ace_main {
        name = "ACE3 - main";
        units[] = {};
        weapons[] = {};
        requiredVersion = 2.10;
        requiredAddons[] = {"cba_main"};
        author = "$STR_ace_common_ACETeam";
        url = "$STR_ace_main_URL";
        version = 3.15; versionStr = "3.15.2.69"; versionAr[] = {3,15,2,69};
    };

    class acex_main: ace_main { 
        units[] = {};
        weapons[] = {};
    };
};

class CfgMods {
    class ace {
        dir = "@ace";
        name = "Advanced Combat Environment 3";
        picture = "A3\Ui_f\data\Logos\arma3_expansion_alpha_ca";
        hidePicture = "true";
        hideName = "true";
        actionName = "Website";
        action = "$STR_ace_main_URL";
        description = "Issue Tracker: https://github.com/acemod/ACE3/issues";
    };
};


class CfgSettings {
    class CBA {
        class Versioning {
            class ACE {
                class dependencies {
                    
                    CBA[] = {"cba_main", {3,15,7}, "(true)"};

                    
                    compat_rhs_afrf3[] = {"ace_compat_rhs_afrf3", {3,15,2,69}, "isClass (configFile >> 'CfgPatches' >> 'rhs_main')"};
                    compat_rhs_usf3[] = {"ace_compat_rhs_usf3", {3,15,2,69}, "isClass (configFile >> 'CfgPatches' >> 'rhsusf_main')"};
                    compat_rhs_gref3[] = {"ace_compat_rhs_gref3", {3,15,2,69}, "isClass (configFile >> 'CfgPatches' >> 'rhsgref_main')"};
                    compat_rhs_saf3[] = {"ace_compat_rhs_saf3", {3,15,2,69}, "isClass (configFile >> 'CfgPatches' >> 'rhssaf_main')"};

                    
                    ace_compat_sog[] = {"ace_compat_sog", {3,15,2,69}, "isClass (configFile >> 'CfgPatches' >> 'data_f_vietnam')"};
                    ace_compat_gm[] = {"ace_compat_gm", {3,15,2,69}, "isClass (configFile >> 'CfgPatches' >> 'gm_core')"};
                };
            };
        };
    };
};
class CfgFactionClasses {
    class NO_CATEGORY;
    class ACE: NO_CATEGORY {
        displayName = "ACE";
        priority = 2;
        side = 7;
    };
    class ACE_Logistics: ACE {
        displayName = "$STR_ace_main_Category_Logistics";
    };
};
class CfgVehicleClasses {
    class ACE_Logistics_Items {
        displayName = "$STR_ace_main_Category_Logistics";
    };
};
class CfgEditorSubcategories {
    class ace_main_subcategory {
        displayName = "$STR_ace_main_Category_Logistics";
    };
};
