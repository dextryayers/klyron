#ifndef QUICKJS_WRAPPER_H
#define QUICKJS_WRAPPER_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct quickjs_engine quickjs_engine;

quickjs_engine* quickjs_init(void);
void quickjs_destroy(quickjs_engine* engine);

char* quickjs_eval(quickjs_engine* engine, const char* code);
char* quickjs_execute_script(quickjs_engine* engine, const char* filename, const char* source);
char* quickjs_execute_module(quickjs_engine* engine, const char* filename, const char* source);

char* quickjs_get_global(quickjs_engine* engine, const char* key);
int   quickjs_set_global(quickjs_engine* engine, const char* key, const char* value);

char* quickjs_call_function(quickjs_engine* engine, const char* name, const char** args, int argc);

unsigned char* quickjs_create_snapshot(quickjs_engine* engine, size_t* out_len);
int            quickjs_load_snapshot(quickjs_engine* engine, const unsigned char* data, size_t len);

const char* quickjs_last_error(quickjs_engine* engine);

void quickjs_free_string(char* s);
void quickjs_free_buffer(unsigned char* buf);

#ifdef __cplusplus
}
#endif

#endif
