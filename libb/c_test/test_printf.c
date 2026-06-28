#include <stddef.h>
#include <stdint.h>

extern void printf(const char *fmt, ...);

int main() {
  printf("Test decimal: %d\n", (int64_t)123456);
  printf("Test negative decimal: %d\n", (int64_t)-7890);
  printf("Test octal: %o\n", (int64_t)83); // Expect: 123
  printf("Test character: %c\n", (int64_t)'A');
  printf("Test string: %s\n", "Hello, world!");
  printf("Test percent: %%\n");
  printf("Test mixed: %d %o %c %s %%\n", (int64_t)42, (int64_t)42, (int64_t)'X',
         "embedded string");
  printf("Test INT64_MIN: %d\n", (int64_t)-9223372036854775807LL - 1LL);
  return 0;
}
