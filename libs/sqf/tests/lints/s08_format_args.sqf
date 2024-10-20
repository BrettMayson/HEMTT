format ["%1", 1];
format x;
format [x, y];
format ["  • %1", 1];
format []; // empty array
format ["%1", 1, 2, 3]; // unused args
format ["%1%2", 1]; // undefined tokens
format ["%5", 1, 2 ,3 ,4, 5]; // skipped tokens
formatText ["me too %1"];
