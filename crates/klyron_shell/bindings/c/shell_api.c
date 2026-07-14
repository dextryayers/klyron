#include "shell_api.h"
#include <stdio.h>

int klyron_shell_execute(void) {
    printf("[shell] execute\n");
    return 0;
}

void klyron_shell_serve(const char* addr) {
    printf("[shell] serving on %s\n", addr);
}
