
class CfgSettings {
    class CBA {
        class Versioning {
            class ACE {
                class dependencies {
                    //ACE will hard exit if this is missing
                    CBA[] = {"cba_main", REQUIRED_CBA_VERSION, "(true)"};

                    //Warnings for missing RHS compat pbos
                    compat_rhs_afrf3[] = {"ace_compat_rhs_afrf3", {VERSION_AR}, "isClass (configFile >> 'CfgPatches' >> 'rhs_main')"};
                    compat_rhs_usf3[] = {"ace_compat_rhs_usf3", {VERSION_AR}, "isClass (configFile >> 'CfgPatches' >> 'rhsusf_main')"};
                    compat_rhs_gref3[] = {"ace_compat_rhs_gref3", {VERSION_AR}, "isClass (configFile >> 'CfgPatches' >> 'rhsgref_main')"};
                    compat_rhs_saf3[] = {"ace_compat_rhs_saf3", {VERSION_AR}, "isClass (configFile >> 'CfgPatches' >> 'rhssaf_main')"};

                    //Warnings for missing DLC compat
                    ace_compat_sog[] = {"ace_compat_sog", {VERSION_AR}, "isClass (configFile >> 'CfgPatches' >> 'data_f_vietnam')"};
                    ace_compat_gm[] = {"ace_compat_gm", {VERSION_AR}, "isClass (configFile >> 'CfgPatches' >> 'gm_core')"};
                };
            };
        };
    };
};
