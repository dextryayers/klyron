#include "compat_api.h"
#include <stdio.h>

int klyron_compat_execute(void) {
    printf("[compat] execute\n");
    return 0;
}

void klyron_compat_serve(const char* addr) {
    printf("[compat] serving on %s\n", addr);
}
