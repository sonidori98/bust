#include <stdint.h>

extern void ctime(const int64_t *time_vec, int64_t *date);
extern void printf(const char *fmt, ...);

int main() {
  char date[16];

  // 1700000000 -> Nov 14, 2023 22:13:20 UTC
  int64_t t1 = 1700000000;
  ctime(&t1, (int64_t *)date);
  printf("T1 (1700000000): %s\n", date);

  // 0 -> Jan 1, 1970 00:00:00 UTC
  int64_t t2 = 0;
  ctime(&t2, (int64_t *)date);
  printf("T2 (0): %s\n", date);

  // -1 -> Dec 31, 1969 23:59:59 UTC
  int64_t t3 = -1;
  ctime(&t3, (int64_t *)date);
  printf("T3 (-1): %s\n", date);

  return 0;
}
