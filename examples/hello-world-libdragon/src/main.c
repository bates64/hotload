#include <stdio.h>

#include <libdragon.h>

// try changing this
int value = 0x1234;

void update(void) {
    printf("value = %X\n", value);
}

int main(void) {
    console_init();

    debug_init_usblog();
    console_set_debug(true);

    while (1) {
        update();
    }
}
