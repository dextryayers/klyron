#include "api.h"
#include <stdlib.h>
#include <string.h>

klyron_runtime_result_t* klyron_runtime_process(const char* input) {
  klyron_runtime_result_t* result = (klyron_runtime_result_t*)malloc(sizeof(klyron_runtime_result_t));
  if (!result) return NULL;
  result->success = true;
  result->data = strdup("processed");
  result->error = NULL;
  return result;
}

const char* klyron_runtime_version(void) {
  return "klyron_runtime 0.1.0";
}

bool klyron_runtime_ping(void) {
  return true;
}

void klyron_runtime_result_free(klyron_runtime_result_t* result) {
  if (result) {
    free(result->data);
    free(result->error);
    free(result);
  }
}
