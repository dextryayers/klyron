#ifndef KLYRON_RUNTIME_BINDINGS_FFI_H
#define KLYRON_RUNTIME_BINDINGS_FFI_H

#include "types.h"

int klyron_runtime_ffi_init(void);
const char* klyron_runtime_ffi_version(void);
char* klyron_runtime_ffi_process(const char* input);
void klyron_runtime_ffi_free_string(char* s);

#endif
