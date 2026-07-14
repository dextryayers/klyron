#include "docker_api.h"
#include <stdio.h>

int klyron_docker_execute(void) {
    printf("[docker] execute\n");
    return 0;
}

void klyron_docker_serve(const char* addr) {
    printf("[docker] serving on %s\n", addr);
}
