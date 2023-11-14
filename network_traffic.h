#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct ProcessPacketLength {
  uint32_t pid;
  uintptr_t upload_length;
  uintptr_t download_length;
} ProcessPacketLength;

typedef struct ProcessStatisticsFFI {
  uintptr_t length;
  const struct ProcessPacketLength *list;
  uint64_t total_upload;
  uint64_t total_download;
} ProcessStatisticsFFI;

void take(void (*f)(struct ProcessStatisticsFFI));

void free_data(struct ProcessStatisticsFFI statistics);
