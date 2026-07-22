#import <Foundation/Foundation.h>
#import <Security/Security.h>
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
    SecCodeRef code = NULL;
    CFDictionaryRef signing_info = NULL;
    OSStatus status = SecCodeCopySelf(kSecCSDefaultFlags, &code);
    if (status == errSecSuccess) {
        status = SecCodeCopySigningInformation(code, kSecCSSigningInformation, &signing_info);
    }
    NSString *team_identifier = signing_info == NULL
        ? nil
        : (__bridge NSString *)CFDictionaryGetValue(signing_info, kSecCodeInfoTeamIdentifier);
    if (team_identifier.length == 0) {
        if (error_buffer != NULL && error_buffer_length > 0) {
            strlcpy(error_buffer,
                    "Fan actuation registration requires an Apple Development or Developer ID signed app; ad-hoc signatures cannot register the privileged daemon.",
                    error_buffer_length);
        }
        if (signing_info != NULL) CFRelease(signing_info);
        if (code != NULL) CFRelease(code);
        return false;
    }
    if (signing_info != NULL) CFRelease(signing_info);
    if (code != NULL) CFRelease(code);

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
