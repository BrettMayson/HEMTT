#pragma hemtt flag pe26_ignore_unsupported_builtin
#if __GAME_VER_MIN__ == 22
version = 22;
#else
version = 0;
#endif
#ifdef __GAME_BUILD__
build = 1;
#endif
value = "yes";
