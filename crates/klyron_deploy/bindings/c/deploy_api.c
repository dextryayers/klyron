#include "deploy_api.h"
#include <stdio.h>

int klyron_deploy_execute(void) {
    printf("[deploy] execute\n");
    return 0;
}

void klyron_deploy_serve(const char* addr) {
    printf("[deploy] serving on %s\n", addr);
}
