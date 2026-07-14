#ifndef KLYRON_ENGINE_BINDINGS_API_H
#define KLYRON_ENGINE_BINDINGS_API_H

#include "types.h"
#include "config.h"

klyron_engine_result_t* klyron_engine_process(const char* input);
const char* klyron_engine_version(void);
bool klyron_engine_ping(void);
void klyron_engine_result_free(klyron_engine_result_t* result);

#endif
