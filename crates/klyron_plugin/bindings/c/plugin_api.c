#include "plugin_api.h"
#include <stdio.h>

int klyron_plugin_execute(void) {
    // TODO: implement
    return 0;
}

void klyron_plugin_serve(const char* addr) {
    printf("[plugin] serving on %s\n", addr);
}
