#ifndef KLYRON_CLI_BINDINGS_FFI_H
#define KLYRON_CLI_BINDINGS_FFI_H

#include "types.h"

int klyron_cli_ffi_init(void);
const char* klyron_cli_ffi_version(void);
char* klyron_cli_ffi_process(const char* input);
void klyron_cli_ffi_free_string(char* s);

#endif
