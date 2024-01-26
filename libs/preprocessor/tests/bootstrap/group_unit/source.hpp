#define QUOTE(var1) #var1

#define GROUP_UNIT(UNIT,SIDE) __GROUP_UNIT(__COUNTER__,UNIT,SIDE)

#define __GROUP_UNIT(COUNTER, UNIT, SIDE) \
    class unit##COUNTER { \
        vehicle = #UNIT; \
        side = SIDE; \
        #if COUNTER == 0 \
        rank = "SERGEANT"; \
        #else \
        #if COUNTER == 2 \
        rank = "SERGEANT"; \
        #else \
        #if COUNTER == 3 \
        rank = "CORPORAL"; \
        #else \
        #if COUNTER == 5 \
        rank = "SERGEANT"; \
        #else \
        #if COUNTER == 6 \
        rank = "CORPORAL"; \
        #else \
        rank = "PRIVATE"; \
        #endif \
        #endif \
        #endif \
        #endif \
        #endif \
        position[] = { \
            QUOTE(5 * floor ((COUNTER + 1) / 2) * (-1 ^ (COUNTER + 1))), \
            QUOTE(-5 * floor ((COUNTER + 1) / 2)), \
            0 \
        }; \
    }

GROUP_UNIT(B_Soldier_F,WEST);
GROUP_UNIT(B_Soldier_F,WEST);
