#include <fcntl.h>
#include <stdint.h>
#include <unistd.h>

struct sgttyb {
  char sg_ispeed;
  char sg_ospeed;
  char sg_erase;
  char sg_kill;
  int sg_flags;
};

extern int64_t gtty(int64_t file, struct sgttyb *buf);
extern int64_t stty(int64_t file, struct sgttyb *buf);

int main() {
  struct sgttyb buf;
  int fd;

  /* Test 1: invalid fd should return -1 */
  if (gtty(-1, &buf) != -1)
    return 'F';
  if (stty(-1, &buf) != -1)
    return 'G';

  /* Test 2: non-tty fd (pipe) should return -1 */
  int p[2];
  if (pipe(p) < 0)
    return 'H';
  if (gtty(p[0], &buf) != -1 || gtty(p[1], &buf) != -1)
    return 'I';
  if (stty(p[0], &buf) != -1 || stty(p[1], &buf) != -1)
    return 'J';
  close(p[0]);
  close(p[1]);

  /* Test 3: tty fd (if available) - roundtrip gtty then stty */
  fd = open("/dev/tty", O_RDWR);
  if (fd < 0)
    fd = open("/dev/console", O_RDWR);
  if (fd >= 0) {
    struct sgttyb saved, restored, readback;
    if (gtty(fd, &saved) != 0) {
      close(fd);
      return 'K';
    }
    restored = saved;
    if (stty(fd, &restored) != 0) {
      close(fd);
      return 'L';
    }
    if (gtty(fd, &readback) != 0) {
      close(fd);
      return 'M';
    }
    if (readback.sg_flags != saved.sg_flags) {
      close(fd);
      return 'N';
    }
    close(fd);
  }

  /* Test 4: verify struct sgttyb size is 8 */
  if (sizeof(struct sgttyb) != 8)
    return 'O';

  /* Test 5: toggle ECHO flag on tty and restore */
  fd = open("/dev/tty", O_RDWR);
  if (fd < 0)
    fd = open("/dev/console", O_RDWR);
  if (fd >= 0) {
    struct sgttyb orig;
    if (gtty(fd, &orig) != 0) {
      close(fd);
      return 'P';
    }
    struct sgttyb mod = orig;
    mod.sg_flags ^= 010;
    if (stty(fd, &mod) != 0) {
      close(fd);
      return 'Q';
    }
    if (stty(fd, &orig) != 0) {
      close(fd);
      return 'R';
    }
    close(fd);
  }

  write(1, "OK\n", 3);
  return 0;
}
