#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <sched.h>
#include <sys/neutrino.h>
#include <sys/iomsg.h>
#include <time.h>

static int chid;

static void set_prio(int prio) {
    struct sched_param sp = { .sched_priority = prio };
    pthread_setschedparam(pthread_self(), SCHED_RR, &sp);
}

static void burn_ms(int ms) {
    struct timespec s, n;
    clock_gettime(CLOCK_MONOTONIC, &s);
    do {
        clock_gettime(CLOCK_MONOTONIC, &n);
    } while ((n.tv_sec - s.tv_sec) * 1000 + (n.tv_nsec - s.tv_nsec) / 1000000 < ms);
}

/* low priority server: does 20 ms of work per request */
static void *server(void *arg) {
    set_prio(10);
    int rcvid;
    char msg[16];
    for (;;) {
        rcvid = MsgReceive(chid, msg, sizeof(msg), NULL);
        if (rcvid > 0) {
            burn_ms(20);
            MsgReply(rcvid, 0, "ok", 3);
        }
    }
    return NULL;
}

/* high priority client: sends a request every 100 ms */
static void *client(void *arg) {
    set_prio(50);
    int coid = ConnectAttach(0, 0, chid, _NTO_SIDE_CHANNEL, 0);
    char reply[16];
    for (int i = 0; i < 20; i++) {
        MsgSend(coid, "hi", 3, reply, sizeof(reply));
        struct timespec ts = { 0, 100 * 1000 * 1000 };
        nanosleep(&ts, NULL);
    }
    return NULL;
}

int main(void) {
    chid = ChannelCreate(0);
    pthread_t s, c;
    pthread_create(&s, NULL, server, NULL);
    pthread_create(&c, NULL, client, NULL);
    pthread_join(c, NULL);
    printf("done\n");
    return 0;
}
