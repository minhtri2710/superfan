#pragma once

#include <stdbool.h>
#include <stddef.h>

int superfan_fan_daemon_status(void);
bool superfan_register_fan_daemon(char *error_buffer, size_t error_buffer_length);
void superfan_open_login_items_settings(void);
