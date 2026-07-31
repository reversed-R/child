#include <stdio.h>

int x = 0;
extern int y;

int main(int argc, char *argv[]) {
  printf("Hello, World! x=%d, y=%d\n", x, y);
  return 0;
}
