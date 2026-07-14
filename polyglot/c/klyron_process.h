#ifndef KLYRON_PROCESS_H
#define KLYRON_PROCESS_H

#include "klyron_types.h"

int klyron_process_exec(const char *cmd, char *const argv[]);
klyron_process_result_t klyron_process_capture(const char *cmd);
void klyron_process_free_result(klyron_process_result_t *result);

#endif
