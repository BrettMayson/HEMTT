diag_log format ["setMission: %1", getMissionPath ""];
diag_log format ["response: %1", "hemtt_comm" callExtension ["mission", [getMissionPath ""]]];
