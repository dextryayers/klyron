#ifndef JSC_WRAPPER_H
#define JSC_WRAPPER_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct jsc_engine jsc_engine;

jsc_engine* jsc_init(void);
void jsc_destroy(jsc_engine* engine);

char* jsc_eval(jsc_engine* engine, const char* code);
char* jsc_execute_script(jsc_engine* engine, const char* filename, const char* source);
char* jsc_execute_module(jsc_engine* engine, const char* filename, const char* source);

char* jsc_get_global(jsc_engine* engine, const char* key);
int   jsc_set_global(jsc_engine* engine, const char* key, const char* value);

char* jsc_call_function(jsc_engine* engine, const char* name, const char** args, int argc);

unsigned char* jsc_create_snapshot(jsc_engine* engine, size_t* out_len);
int            jsc_load_snapshot(jsc_engine* engine, const unsigned char* data, size_t len);

const char* jsc_last_error(jsc_engine* engine);

void jsc_free_string(char* s);
void jsc_free_buffer(unsigned char* buf);

#ifdef __cplusplus
}
#endif

#endif
