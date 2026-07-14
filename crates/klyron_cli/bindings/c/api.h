#ifndef KLYRON_CLI_BINDINGS_API_H
#define KLYRON_CLI_BINDINGS_API_H

#include "types.h"
#include "config.h"

klyron_cli_result_t* klyron_cli_process(const char* input);
const char* klyron_cli_version(void);
bool klyron_cli_ping(void);
void klyron_cli_result_free(klyron_cli_result_t* result);

#endif
