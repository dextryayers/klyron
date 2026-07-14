#ifndef KLYRON_UPDATER_BINDINGS_API_H
#define KLYRON_UPDATER_BINDINGS_API_H

#include "types.h"
#include "config.h"

klyron_updater_result_t* klyron_updater_process(const char* input);
const char* klyron_updater_version(void);
bool klyron_updater_ping(void);
void klyron_updater_result_free(klyron_updater_result_t* result);

#endif
