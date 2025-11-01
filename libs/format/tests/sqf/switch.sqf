switch (floor random 6) do {
    case 1;
    case 3;
    case 5: {
        systemChat "Odd case selected";
    };
    case 0;
    case 2;
    case 4: {
        systemChat "Even case selected";
    };
    default {
        systemChat "Default case selected";
    };
};
