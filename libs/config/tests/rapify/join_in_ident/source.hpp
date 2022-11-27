#define DOUBLES(a,b) a##b

class thing {
	DOUBLES(hello,world) = "test";	
};
