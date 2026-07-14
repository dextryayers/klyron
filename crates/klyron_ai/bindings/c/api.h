#ifndef KLYRON_AI_BINDINGS_API_H
#define KLYRON_AI_BINDINGS_API_H

#include "types.h"
#include "config.h"

klyron_ai_result_t* klyron_ai_process(const char* input);
const char* klyron_ai_version(void);
bool klyron_ai_ping(void);
void klyron_ai_result_free(klyron_ai_result_t* result);

#endif
