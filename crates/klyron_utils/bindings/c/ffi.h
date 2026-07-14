#ifndef KLYRON_UTILS_BINDINGS_FFI_H
#define KLYRON_UTILS_BINDINGS_FFI_H

#include "types.h"

int klyron_utils_ffi_init(void);
const char* klyron_utils_ffi_version(void);
char* klyron_utils_ffi_process(const char* input);
void klyron_utils_ffi_free_string(char* s);

#endif
