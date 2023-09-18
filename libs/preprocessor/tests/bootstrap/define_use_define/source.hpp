#define QUOTE(s) #s

#define HELLO(name) QUOTE(Hello name)

value = HELLO(Brett);

#define NAME Brett
#define HELLO Hello NAME

value = QUOTE(HELLO);
