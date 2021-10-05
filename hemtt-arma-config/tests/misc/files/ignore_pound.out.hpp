class CfgVehicles {
    class something {
		hiddenSelectionsTextures[] = {
			"#(argb,8,8,3)color(0,0,0,0.0,co)",
			"#(argb,8,8,3)color(0,0,0,0.0,co)",
			"#(argb,8,8,3)color(0,0,0,0.0,co)",
			"#(argb,8,8,3)color(0,0,0,0.0,co)",
			"#(argb,8,8,3)color(0,0,0,0.0,co)",
			"#(argb,8,8,3)color(0,0,0,0.0,co)",
			"#(argb,8,8,3)color(0,0,0,0.0,co)",
			"#(argb,8,8,3)color(0,0,0,0.0,co)",
			"#(argb,8,8,3)color(0,0,0,0.0,co)",
			"a3\props_f_exp\military\camps\data\tripodscreen_01_co.paa"};
		class ACE_Actions {
			class ACE_MainActions {
				class spectator_open {
					displayName = "Spectator";
					condition = "spectator_allowed";
					statement = "[true, false] call ace_spectator_fnc_setSpectator";
					icon = "\a3\3den\data\cfg3den\camera\cameratexture_ca.paa";
				};
			};
		};
	};
};
