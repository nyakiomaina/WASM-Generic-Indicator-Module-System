#![feature(prelude_2024)]

use core::prelude::rust_2024::derive;

#[derive(PartialEq, Clone, Debug)]
struct LedColor(i32, i32, i32);

const SENSOR_BATTERY: i32 = 20;

const OFF:LedColor = LedColor(0, 0, 0);
const YELLOW: LedColor = LedColor(255, 255, 0);
const GREEN: LedColor = LedColor(0, 255, 0);
const RED: LedColor = LedColor(255, 0, 0);
const PCT_PER_PIXEL: f64 = 12.5_f64;

unsafe extern "C" {
    fn set_led(led_index: i32, r: i32, g: i32, b: i32);
}

#[unsafe(no_mangle)]
pub extern "C" fn sensor_update(sensor_id: i32, sensor_value: f64) -> f64 {
    if sensor_id == SENSOR_BATTERY {
        set_leds(get_led_values(sensor_value));
    }
    sensor_value
}

#[unsafe(no_mangle)]
pub extern "C" fn apply(_frame: i32) {
    // no-op
}

fn get_led_values(battery_remaining: f64) -> [LedColor; 8] {
    let mut arr: [LedColor; 8] = [OFF,OFF,OFF,OFF,OFF,OFF,OFF,OFF,];
    let lit = (battery_remaining / PCT_PER_PIXEL).ceil();

    // 0 - 20 : Red
    // 21 - <50 : Yellow
    // 51 - 100 : Green
    let color = if 0.0 <= battery_remaining &&
        battery_remaining <= 20.0 {
        RED
    } else if battery_remaining > 20.0 && battery_remaining < 50.0 {
        YELLOW
    } else {
        GREEN
    };
    for idx in 0..lit as usize {
        arr[idx] = color.clone();
    }
    arr
}

fn set_leds(value: [LedColor; 8]) {
    for x in 0..8 {
        let LedColor(r, g, b) = value[x];
        unsafe {
            set_led(x as i32, r, g, b);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{OFF, YELLOW, GREEN, RED, get_led_values};

    #[test]
    fn test_0_pct() {
        assert_eq!(get_led_values(0.0),
                   [OFF, OFF, OFF, OFF, OFF, OFF, OFF, OFF]);
    }

    #[test]
    fn test_15_pct() {
        assert_eq!(get_led_values(15.0),
                   [RED, RED, OFF, OFF, OFF, OFF, OFF, OFF]);
    }

    #[test]
    fn test_49_pct() {
        assert_eq!(get_led_values(49.0),
                   [YELLOW, YELLOW, YELLOW, YELLOW, OFF, OFF, OFF, OFF]);
    }

    #[test]
    fn test_75_pct() {
        assert_eq!(get_led_values(75.0),
                   [GREEN, GREEN, GREEN, GREEN, GREEN, GREEN, OFF, OFF]);
    }

    #[test]
    fn test_100_pct() {
        assert_eq!(get_led_values(100.0),
                   [GREEN, GREEN, GREEN, GREEN, GREEN, GREEN, GREEN, GREEN]);
    }
}