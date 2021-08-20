mod coverage;

pub use coverage::inject;

pub const ARMA_STARTUP: &[&'static str] = &["-window", "-noSplash", "-skipIntro", "-name=\"hemtt_tests\"", "-noPause", "-showScriptErrors", "-debug", "-mod=\"F:\\SteamLibrary\\steamapps\\common\\Arma 3\\!Workshop\\@CBA_A3;P:\\arma\\hemtt-exp\\hemtt-tests\\src\\mod\"", r#"-init=playMission['','\hemtt\tests\addons\main\missions\sp_blufor_rifleman.Stratis',true]"#];
