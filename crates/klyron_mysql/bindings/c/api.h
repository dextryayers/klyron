#ifndef KLYRON_MYSQL_BINDINGS_API_H
#define KLYRON_MYSQL_BINDINGS_API_H

#include "types.h"
#include "config.h"

klyron_mysql_result_t* klyron_mysql_process(const char* input);
const char* klyron_mysql_version(void);
bool klyron_mysql_ping(void);
void klyron_mysql_result_free(klyron_mysql_result_t* result);

#endif
