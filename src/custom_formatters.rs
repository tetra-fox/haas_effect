use std::sync::Arc;

pub fn v2s_f32_ms_then_s(digits: usize) -> Arc<dyn Fn(f32) -> String + Send + Sync> {
    Arc::new(move |value| {
        if value < 100. {
            format!("{:.1} ms", value)
        } else if value < 1000. {
            format!("{:.0} ms", value)
        } else {
            format!(
                "{:.digits$} s",
                value as f32 / 1000.0,
                digits = digits.max(2)
            )
        }
    })
}

pub fn s2v_f32_ms_then_s() -> Arc<dyn Fn(&str) -> Option<f32> + Send + Sync> {
    Arc::new(|string| {
        let mut string = string.trim_end();

        let is_ms = string.ends_with("ms") || string.ends_with("MS");
        let is_sec = string.ends_with("s") || string.ends_with("S");

        if is_ms {
            string = string.trim_end_matches(&[' ', 'm', 'M', 's', 'S']);

            match string.parse::<f32>() {
                Ok(num) => Some(num),
                Err(_) => None,
            }
        } else if is_sec {
            string = string.trim_end_matches(&[' ', 's', 'S']);

            match string.parse::<f32>() {
                Ok(num) => Some(num / 1000.0),
                Err(_) => None,
            }
        } else {
            None
        }
    })
}
