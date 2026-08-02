#include <stdio.h>

int x = 0;
extern int y;

int main(int argc, char *argv[]) {
  printf("Hello, World! x=%d, y=%d, x+y=%d\n", x, y, x + y);
  return x + y;
}
