pub fn hsv_from_rgb(rgb: (f64, f64, f64)) -> (f64, f64, f64) {
    let r = rgb.0 / 255.0;
    let g = rgb.1 / 255.0;
    let b = rgb.2 / 255.0;

    let min = r.min(g.min(b));
    let max = r.max(g.max(b));
    let delta = max - min;

    let v = max;
    let s = match max > 1e-3 {
        true => delta / max,
        false => 0.0,
    };
    let h = match delta == 0.0 {
        true => 0.0,
        false => {
            if r == max {
                (g - b) / delta
            } else if g == max {
                2.0 + (b - r) / delta
            } else {
                4.0 + (r - g) / delta
            }
        }
    };
    let h2 = ((h * 60.0) + 360.0) % 360.0;

    (h2, s, v)
}