#include <stdint.h>

extern void putchar(int64_t chr);
extern void printn(int64_t n, int64_t b);

int main() {
  printn(12345, 10);
  putchar('\n');

  printn(-6789, 10);
  putchar('\n');

  printn(0, 10);
  putchar('\n');

  printn(83, 8);
  putchar('\n');

  printn((int64_t)-9223372036854775807LL - 1LL, 10);
  putchar('\n');

  return 0;
}
