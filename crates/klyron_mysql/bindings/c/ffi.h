#ifndef KLYRON_MYSQL_BINDINGS_FFI_H
#define KLYRON_MYSQL_BINDINGS_FFI_H

#include "types.h"

int klyron_mysql_ffi_init(void);
const char* klyron_mysql_ffi_version(void);
char* klyron_mysql_ffi_process(const char* input);
void klyron_mysql_ffi_free_string(char* s);

#endif
