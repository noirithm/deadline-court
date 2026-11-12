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

static void sleep_ms(int ms) {
    struct timespec ts = { ms / 1000, (ms % 1000) * 1000 * 1000 };
    nanosleep(&ts, NULL);
}

static int cur_prio(void) {
    struct sched_param sp;
    int pol;
    pthread_getschedparam(pthread_self(), &pol, &sp);
    return sp.sched_priority;
}

/* low priority server: 20 ms of work per request */
static void *server(void *arg) {
    set_prio(10);
    char msg[16];
    int n = 0;
    printf("server start prio %d\n", cur_prio());
    for (;;) {
        int rcvid = MsgReceive(chid, msg, sizeof(msg), NULL);
        if (rcvid > 0) {
            int during = cur_prio();
            burn_ms(20);
            MsgReply(rcvid, 0, "ok", 3);
            if (n++ < 3)
                printf("req %d: prio during=%d after reply=%d\n", n, during, cur_prio());
        }
    }
    return NULL;
}

/* high priority client: one request every 100 ms */
static void *client(void *arg) {
    set_prio(50);
    int coid = ConnectAttach(0, 0, chid, _NTO_SIDE_CHANNEL, 0);
    char reply[16];
    for (int i = 0; i < 20; i++) {
        MsgSend(coid, "hi", 3, reply, sizeof(reply));
        sleep_ms(100);
    }
    return NULL;
}

/* medium priority hog: 30 ms of CPU every 100 ms, between server and client priority */
static void *hog(void *arg) {
    set_prio(30);
    for (;;) {
        burn_ms(30);
        sleep_ms(70);
    }
    return NULL;
}

int main(void) {
    chid = ChannelCreate(0);
    pthread_t s, c, h;
    pthread_create(&s, NULL, server, NULL);
    pthread_create(&h, NULL, hog, NULL);
    pthread_create(&c, NULL, client, NULL);
    pthread_join(c, NULL);
    printf("done\n");
    return 0;
}
