#include "api.h"
#include <stdlib.h>
#include <string.h>

klyron_mysql_result_t* klyron_mysql_process(const char* input) {
  klyron_mysql_result_t* result = (klyron_mysql_result_t*)malloc(sizeof(klyron_mysql_result_t));
  if (!result) return NULL;
  result->success = true;
  result->data = strdup("processed");
  result->error = NULL;
  return result;
}

const char* klyron_mysql_version(void) {
  return "klyron_mysql 0.1.0";
}

bool klyron_mysql_ping(void) {
  return true;
}

void klyron_mysql_result_free(klyron_mysql_result_t* result) {
  if (result) {
    free(result->data);
    free(result->error);
    free(result);
  }
}
