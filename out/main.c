#include <stdio.h>

// #include "../libruntime/libruntime.h"
#include "./out.h"

int main() {
  printf("Hello!\n");

  struct CLASS_2 c;
  CLASS_2____init_(&c);

  int32_t input = 170;
  printf("Input: %d\n", input);

  int32_t res = CLASS_2__getMax2(&c, input);
  printf("Result: %d\n", res);

  return 0;
}