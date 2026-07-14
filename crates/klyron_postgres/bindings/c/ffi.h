#ifndef KLYRON_POSTGRES_BINDINGS_FFI_H
#define KLYRON_POSTGRES_BINDINGS_FFI_H

#include "types.h"

int klyron_postgres_ffi_init(void);
const char* klyron_postgres_ffi_version(void);
char* klyron_postgres_ffi_process(const char* input);
void klyron_postgres_ffi_free_string(char* s);

#endif
