#ifndef KLYRON_SQLITE_BINDINGS_FFI_H
#define KLYRON_SQLITE_BINDINGS_FFI_H

#include "types.h"

int klyron_sqlite_ffi_init(void);
const char* klyron_sqlite_ffi_version(void);
char* klyron_sqlite_ffi_process(const char* input);
void klyron_sqlite_ffi_free_string(char* s);

#endif
