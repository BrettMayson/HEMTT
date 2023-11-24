// COMPONENT should be defined in the script_component.hpp and included BEFORE this hpp

#define MAINPREFIX z
#define PREFIX abe

// MINIMAL required version for the Mod. Components can specify others..
#define REQUIRED_VERSION 2.10
#define REQUIRED_CBA_VERSION {3,15,7}

// #ifdef COMPONENT_BEAUTIFIED
//     #define COMPONENT_NAME QUOTE(ACE3 - COMPONENT_BEAUTIFIED)
// #else
    #define COMPONENT_NAME QUOTE(ACE3 - COMPONENT)
// #endif
