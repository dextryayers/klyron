#include "adapter_api.h"
#include <stdio.h>

int klyron_adapter_execute(void) {
    printf("[adapter] execute\n");
    return 0;
}

void klyron_adapter_serve(const char* addr) {
    printf("[adapter] serving on %s\n", addr);
}
