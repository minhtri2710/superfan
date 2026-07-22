#import <Foundation/Foundation.h>
#import <ServiceManagement/ServiceManagement.h>
#include <stdbool.h>
#include <string.h>

static NSString *const SuperFanDaemonPlist = @"com.superfan.fan-actuation.plist";

int superfan_fan_daemon_status(void) {
    // SMAppService is the macOS 13+ interface for bundled LaunchDaemons.
    // Source: https://developer.apple.com/documentation/servicemanagement/smappservice
    SMAppService *service = [SMAppService daemonServiceWithPlistName:SuperFanDaemonPlist];
    return (int)service.status;
}

bool superfan_register_fan_daemon(char *error_buffer, size_t error_buffer_length) {
    SMAppService *service = [SMAppService daemonServiceWithPlistName:SuperFanDaemonPlist];
    NSError *error = nil;
    BOOL registered = [service registerAndReturnError:&error];
    if (!registered && error_buffer != NULL && error_buffer_length > 0) {
        const char *message = error.localizedDescription.UTF8String;
        if (message != NULL) {
            strlcpy(error_buffer, message, error_buffer_length);
        }
    }
    return registered;
}

void superfan_open_login_items_settings(void) {
    [SMAppService openSystemSettingsLoginItems];
}
