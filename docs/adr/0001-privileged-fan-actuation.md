---
status: superseded
superseded_by: 0005
---

# Use a privileged launchd service for Fan actuation

SuperFan will replace the setuid `smc-helper` path with a privileged launchd service. The Fan actuation module will use a narrow Unix domain socket protocol authorized for the active console user, validate fan targets against hardware ranges, and restore all fans to System Auto on communication or service failure; installation, updates, restarts, and migration of the old helper remain behind a separate bootstrap seam. This adds macOS lifecycle and signing complexity, but reduces the privileged trust surface, gives runtime actuation one implementation, and supports in-memory adapter tests plus socket integration tests.
