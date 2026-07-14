#ifndef KLYRON_SQLITE_BINDINGS_API_H
#define KLYRON_SQLITE_BINDINGS_API_H

#include "types.h"
#include "config.h"

klyron_sqlite_result_t* klyron_sqlite_process(const char* input);
const char* klyron_sqlite_version(void);
bool klyron_sqlite_ping(void);
void klyron_sqlite_result_free(klyron_sqlite_result_t* result);

#endif
