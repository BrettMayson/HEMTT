format ["%1", 1];
format x;
format [x, y];
format ["  • %1", 1];
format []; // empty array
format ["%1", 1, 2, 3]; // unused args
format ["%1%2", 1]; // undefined tokens
format ["%5", 1, 2 ,3 ,4, 5]; // skipped tokens
formatText ["me too %1"];

format ["%1%%", 100];
format ["%%%1%%", 100];
format ["%%%%%%%%%%%%%%%%"];
format ["this code is 99% bug free"]; // non-escaped
format ["%1%"]; // non-escaped (prioity over unused)
format ["%%1", 1]; // unused args
format ["%%%1", 1];
format ["%%%1%%%2 %% %%%3%%%%", 1, 2, 3];
