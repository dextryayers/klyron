#include "api.h"
#include <stdlib.h>
#include <string.h>

klyron_engine_result_t* klyron_engine_process(const char* input) {
  klyron_engine_result_t* result = (klyron_engine_result_t*)malloc(sizeof(klyron_engine_result_t));
  if (!result) return NULL;
  result->success = true;
  result->data = strdup("processed");
  result->error = NULL;
  return result;
}

const char* klyron_engine_version(void) {
  return "klyron_engine 0.1.0";
}

bool klyron_engine_ping(void) {
  return true;
}

void klyron_engine_result_free(klyron_engine_result_t* result) {
  if (result) {
    free(result->data);
    free(result->error);
    free(result);
  }
}
