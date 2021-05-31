// clang C99, with -O0
int main(int argc, char* argv[]) {
  int x = 5;
  int y = 2;
  int z = 10;
  if (x > y) {
    return z + 1;
  } else {
    return z + 2;
  }
}
