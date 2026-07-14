#include "api.h"
#include <stdlib.h>
#include <string.h>

klyron_utils_result_t* klyron_utils_process(const char* input) {
  klyron_utils_result_t* result = (klyron_utils_result_t*)malloc(sizeof(klyron_utils_result_t));
  if (!result) return NULL;
  result->success = true;
  result->data = strdup("processed");
  result->error = NULL;
  return result;
}

const char* klyron_utils_version(void) {
  return "klyron_utils 0.1.0";
}

bool klyron_utils_ping(void) {
  return true;
}

void klyron_utils_result_free(klyron_utils_result_t* result) {
  if (result) {
    free(result->data);
    free(result->error);
    free(result);
  }
}
