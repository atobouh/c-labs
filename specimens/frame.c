#include <stdio.h>

int twice(int value) {
    int result = value * 2;
    return result;
}

int main(void) {
    int answer = twice(4);
    printf("%d\n", answer);
    return 0;
}
