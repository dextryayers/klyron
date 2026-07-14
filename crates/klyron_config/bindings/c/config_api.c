#include "config_api.h"
#include <stdio.h>

int klyron_config_execute(void) {
    printf("[config] execute\n");
    return 0;
}

void klyron_config_serve(const char* addr) {
    printf("[config] serving on %s\n", addr);
}
