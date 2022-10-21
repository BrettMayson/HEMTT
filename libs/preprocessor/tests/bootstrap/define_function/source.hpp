#define QUOTE(x) #x
#define HELLO(NAME) Hello NAME
#define INTRO(ALPHA, BRAVO) Hello ALPHA, meet BRAVO

#define AUTHOR John

value = QUOTE(HELLO(AUTHOR));
