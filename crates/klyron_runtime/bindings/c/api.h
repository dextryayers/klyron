#ifndef KLYRON_RUNTIME_BINDINGS_API_H
#define KLYRON_RUNTIME_BINDINGS_API_H

#include "types.h"
#include "config.h"

klyron_runtime_result_t* klyron_runtime_process(const char* input);
const char* klyron_runtime_version(void);
bool klyron_runtime_ping(void);
void klyron_runtime_result_free(klyron_runtime_result_t* result);

#endif
