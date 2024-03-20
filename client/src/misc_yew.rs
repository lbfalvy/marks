use std::time::{Duration, SystemTime, UNIX_EPOCH};

use web_sys::js_sys::Date;
use web_sys::js_sys::Math::random;

pub fn u32rand() -> u32 { (random() * u32::MAX as f64).floor() as u32 }
pub fn now() -> SystemTime { UNIX_EPOCH + Duration::from_secs_f64(Date::now() / 1000f64) }
