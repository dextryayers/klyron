#ifndef KLYRON_POSTGRES_BINDINGS_API_H
#define KLYRON_POSTGRES_BINDINGS_API_H

#include "types.h"
#include "config.h"

klyron_postgres_result_t* klyron_postgres_process(const char* input);
const char* klyron_postgres_version(void);
bool klyron_postgres_ping(void);
void klyron_postgres_result_free(klyron_postgres_result_t* result);

#endif
