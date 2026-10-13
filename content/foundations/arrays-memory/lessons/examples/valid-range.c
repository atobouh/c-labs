#include <stddef.h>
#include <stdio.h>

static void describe_range(const int *values, size_t count)
{
    if (count == 0) {
        puts("empty range: no readable index");
        return;
    }

    printf("first index=0 value=%d\n", values[0]);
    printf("last index=%zu value=%d\n", count - 1, values[count - 1]);

    const int *one_past = values + count;
    printf("one-past offset=%td; no value is read\n", one_past - values);
}

int main(void)
{
    const int readings[] = {12, 18, 9};
    describe_range(readings, sizeof readings / sizeof readings[0]);
    describe_range(readings, 0);
    return 0;
}
