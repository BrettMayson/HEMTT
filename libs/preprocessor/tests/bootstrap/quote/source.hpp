#define QUOTE(x) #x
#define TEST 123

class test {
    value = #TEST;
    value = QUOTE(TEST);
};
