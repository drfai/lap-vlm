// Copyright 2026 Lap contributors.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// GetAsyncKeyState reports physical buttons; DOM button 0 is the logical primary.
pub(super) fn primary_button_pressed(swapped: bool, left_state: i16, right_state: i16) -> bool {
    (if swapped { right_state } else { left_state }) < 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_button_tracks_swap_setting_and_release() {
        for swapped in [false, true] {
            assert!(!primary_button_pressed(swapped, 0, 0));
            assert_eq!(primary_button_pressed(swapped, i16::MIN, 0), !swapped);
            assert_eq!(primary_button_pressed(swapped, 0, i16::MIN), swapped);
            assert!(primary_button_pressed(swapped, i16::MIN, i16::MIN));
            // The low bit indicates a previous press, not a currently held button.
            assert!(!primary_button_pressed(swapped, 1, 1));
        }
    }
}
