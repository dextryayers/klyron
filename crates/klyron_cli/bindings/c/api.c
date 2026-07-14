#include "api.h"
#include <stdlib.h>
#include <string.h>

klyron_cli_result_t* klyron_cli_process(const char* input) {
  klyron_cli_result_t* result = (klyron_cli_result_t*)malloc(sizeof(klyron_cli_result_t));
  if (!result) return NULL;
  result->success = true;
  result->data = strdup("processed");
  result->error = NULL;
  return result;
}

const char* klyron_cli_version(void) {
  return "klyron_cli 0.1.0";
}

bool klyron_cli_ping(void) {
  return true;
}

void klyron_cli_result_free(klyron_cli_result_t* result) {
  if (result) {
    free(result->data);
    free(result->error);
    free(result);
  }
}
