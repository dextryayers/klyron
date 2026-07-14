#include "telemetry_api.h"
#include <stdio.h>

int klyron_telemetry_execute(void) {
    printf("[telemetry] execute\n");
    return 0;
}

void klyron_telemetry_serve(const char* addr) {
    printf("[telemetry] serving on %s\n", addr);
}
