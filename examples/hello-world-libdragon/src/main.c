#include <stdio.h>

#include <libdragon.h>

int i = 0;

void update(void) {
    printf("i = %d\n", i);
    i += 1; // try changing this
}

int main(void) {
    console_init();

    debug_init_usblog();
    console_set_debug(true);

    while (1) {
        update();
    }
}
