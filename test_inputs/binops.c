// clang C99, with -O0
int main(int argc, char* argv[]) {
  int x = 24;
  int y = 5;
  int z = x - y;
  int a = z * 40;
  int b = a / 23;
  int c = b & 1;
  int d = c | 3;
  int e = d << (x + 10);
  return e;
}
