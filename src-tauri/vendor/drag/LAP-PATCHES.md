Based on CrabNebula drag 2.1.1 (MIT OR Apache-2.0).
Upstream: https://github.com/crabnebula-dev/drag-rs

Local patches:
- Return a startup error instead of panicking when Windows Shell cannot create a file data object (including SMB/UNC paths).
- Encode GTK file URIs through GIO so spaces, #, % and Unicode paths are valid.
- Pass mouse button 1 to GTK drag_begin_with_coordinates instead of the BUTTON1 modifier mask.
- Keep GTK URI providers until dnd-finished (data transfer complete), rather than removing them at drop-performed (mouse release). Missing cursor devices no longer panic at session completion.
- Validate Windows PIDLs before creating the Shell array and release their allocated memory.
- Cancel pending startup requests and check the live left mouse button immediately before macOS, Windows, or GTK enters native DND.
- Roll back GTK source configuration and URI handlers on every failed or cancelled startup.
- Respect Windows swapped mouse buttons when checking whether the logical primary button is still held.
