#include "utils.h"
#include <string.h>

void klyron_pm_version_str(char* buf, size_t len) {
    if (buf && len > 0) {
        snprintf(buf, len, "%d.%d.%d", 1, 0, 0);
    }
}
