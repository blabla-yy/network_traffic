#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct ProcessPacketLength {
  uint32_t pid;
  uintptr_t upload_length;
  uintptr_t download_length;
} ProcessPacketLength;

typedef struct ProcessStatistics {
  uintptr_t length;
  const struct ProcessPacketLength *list;
  uint64_t elapse_millisecond;
} ProcessStatistics;

void take(void (*f)(struct ProcessStatistics));

void free_array(struct ProcessStatistics stastistics);
