#define ITEM(name) mod_##name

#define MACRO_ADDITEM(ITEM,COUNT) class _xx_##ITEM { \
    name = #ITEM; \
    count = COUNT; \
}

class TransportItems {
    MACRO_ADDITEM(ITEM(painkillers),1);
};
