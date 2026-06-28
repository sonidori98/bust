#include <stddef.h>

extern void execl(const char *path, ...);

int main() {
  execl("/usr/bin/echo", "echo", "execl", "success", NULL);
  return 1;
}
