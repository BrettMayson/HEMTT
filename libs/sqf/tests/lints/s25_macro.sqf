#define GVAR(VAR) test_##VAR

GVAR(activeClients) = [];

if (count GVAR(activeClients) == 0) then {
    systemChat "No active clients.";
};
