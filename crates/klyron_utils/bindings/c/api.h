#ifndef KLYRON_UTILS_BINDINGS_API_H
#define KLYRON_UTILS_BINDINGS_API_H

#include "types.h"
#include "config.h"

klyron_utils_result_t* klyron_utils_process(const char* input);
const char* klyron_utils_version(void);
bool klyron_utils_ping(void);
void klyron_utils_result_free(klyron_utils_result_t* result);

#endif
