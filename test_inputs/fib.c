#include <stdio.h>
#include <stdlib.h>

typedef unsigned long long u64;

u64 fib(u64 a, u64 b, u64 count) {
  if (!count) {
    return b;
  }
  return fib(b, a + b, count - 1);
}

u64 fib_dispatch(u64 fib_index) {
  if (fib_index < 2) {
    return fib_index;
  }
  return fib(0, 1, fib_index - 1);
}

#ifdef NATIVE
// Note to testers: do not call this function directly from WASM embedding. Call
// fib_dispatch instead!
int main(int argc, char** argv) {
  u64 start_cycles = __rdtsc();
  u64 result = fib_dispatch(atoi(argv[1]));
  u64 stop_cycles = __rdtsc();
  printf("Result: %llu\n In %llu cycles\n", result, stop_cycles - start_cycles);
}
#endif
