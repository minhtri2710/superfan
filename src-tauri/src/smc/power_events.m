#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>

typedef void (*PowerEventCallback)(void);

static PowerEventCallback g_will_sleep_cb = NULL;
static PowerEventCallback g_did_wake_cb = NULL;

@interface SuperFanPowerObserver : NSObject
- (void)willSleep:(NSNotification *)note;
- (void)didWake:(NSNotification *)note;
@end

@implementation SuperFanPowerObserver
- (void)willSleep:(NSNotification *)note {
    (void)note;
    if (g_will_sleep_cb) {
        g_will_sleep_cb();
    }
}

- (void)didWake:(NSNotification *)note {
    (void)note;
    if (g_did_wake_cb) {
        g_did_wake_cb();
    }
}
@end

static SuperFanPowerObserver *g_observer = nil;

void superfan_register_sleep_wake_callbacks(PowerEventCallback will_sleep, PowerEventCallback did_wake) {
    g_will_sleep_cb = will_sleep;
    g_did_wake_cb = did_wake;

    if (!g_observer) {
        g_observer = [[SuperFanPowerObserver alloc] init];
        NSNotificationCenter *nc = [[NSWorkspace sharedWorkspace] notificationCenter];
        [nc addObserver:g_observer
               selector:@selector(willSleep:)
                   name:NSWorkspaceWillSleepNotification
                 object:nil];
        [nc addObserver:g_observer
               selector:@selector(didWake:)
                   name:NSWorkspaceDidWakeNotification
                 object:nil];
    }
}
