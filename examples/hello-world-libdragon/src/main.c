#include <stdio.h>

#include <libdragon.h>

void new(void) {
    // dont optimise away
    printf("new\n");
}

int main(void)
{
    console_init();

    debug_init_usblog();
    console_set_debug(true);

    new();

    printf("Hello world!\n");

    while(0xDEADBEEF) {}
}
