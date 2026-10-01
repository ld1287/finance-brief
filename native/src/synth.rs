//! Synthetic data generators used until real sources are wired in.
//! Mirrors the JS `synth_candles` helper from splash (lines ~183–200).



/// 生成 N 根 OHLC 蜡烛 (placeholder for real data)。
pub fn synth_candles(n: usize) -> Vec<crate::model::Candle> {
    let mut out = Vec::with_capacity(n);
    let patterns = [1.0_f64, 0.5, -0.3, -1.0, -0.5, 0.3, 0.8, -0.2];
    let mut price = 100.0_f64;
    for i in 0..n {
        let drift = patterns[i % 8];
        let o = price;
        let c = price + drift;
        let h = o.max(c) + 0.3;
        let l = o.min(c) - 0.3;
        out.push(crate::model::Candle {
            time: i as f64,
            open: o,
            high: h,
            low: l,
            close: c,
            volume: 1000.0,
        });
        price = c;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn synth_candles_count() {
        assert_eq!(synth_candles(120).len(), 120);
    }
    #[test]
    fn synth_candles_high_geq_open() {
        for c in synth_candles(40) {
            assert!(c.high >= c.open.max(c.close));
        }
    }
}
