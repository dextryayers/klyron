#ifndef KLYRON_PROCESS_H
#define KLYRON_PROCESS_H

#include "klyron_types.h"

klyron_process_result_t klyron_process_exec(const char *cmd);
klyron_process_result_t klyron_process_exec_args(const char *cmd, char *const argv[]);
void klyron_process_free_result(klyron_process_result_t *r);
bool klyron_process_which(const char *program, char *out, size_t out_size);

#endif
