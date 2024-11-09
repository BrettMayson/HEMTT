#define HELLO(var1, var2) INNER(Hello, var1##var2)
#define GREET(var1, var2) INNER(Greetings, var1##var2)
#define INNER(var1, var2) var1 var2

value1 = HELLO(John,Smith); // Only inner
value2 = GREET(John, Smith); // Both call and inner
