use crate::data::Candle;
use rand::thread_rng;
use rand_distr::{Distribution, Normal};

// Define the number of trading days in a year as a constant.
const TRADING_DAYS_PER_YEAR: usize = 25;

pub fn daily_returns(candles: &[Candle]) -> Vec<f64> {
    let mut returns: Vec<f64> = Vec::new();
    //candles array has less then two candles we cannot calculate the daily return
    if candles.len() < 2 {
        return returns;
    }
    //loop from item 2 to the end of the list
    for candle in 1..candles.len() {
        let yesterday = &candles[candle - 1];
        let today = &candles[candle];
        //calculate log return
        let ret = (today.close / yesterday.close).ln();
        returns.push(ret);
    }
    returns
}

pub fn calc_stats(returns: &[f64]) -> Option<(f64, f64)> {
    let count = returns.len() as f64;
    if count < 2.0 {
        return None;
    }
    let sum: f64 = returns.iter().sum();
    //average return
    let avr = sum / count;
    //variance -> for each data point, subtract the mean and square result
    //sum of squared diff
    let variance = returns
        .iter()
        .map(|value| {
            let diff = value - avr;
            diff * diff
        })
        .sum::<f64>()
        / (count - 1.0); // sum is a decimal and using n-1 to correct any errors in the small dataset
    //std dev is the squareroot of variance
    let std_dev = variance.sqrt();
    //if values vals exist, return  (Option)
    Some((avr, std_dev))
}
//rf = risk free return, n_sims = numb of simulations
pub fn monte_carlo_sharpe(avr: f64, std_dev: f64, rf: f64, n_sims: usize) -> Vec<f64> {
    // generate random returns, compute Sharpe ratios, return them

    let mut sim_sharpe_r = Vec::with_capacity(n_sims);
    //handle 0 volatility by just returning empty vector since dividng by 0 is not possible  
       if std_dev == 0.0 {
        return sim_sharpe_r;
    }
      // Standard normal distribution to generate random daily returns
    let mut rng = thread_rng();
    let ret_dist = Normal::new(avr, std_dev).unwrap();

    for _ in 0..n_sims {
        //sim returns through this range (one year)
        let sim_ret: Vec<f64> = (0..TRADING_DAYS_PER_YEAR)
            .map(|_| ret_dist.sample(&mut rng))
            .collect();

        //Compute mean & Std dev
        if let Some((avr, std_dev)) = calc_stats(&sim_ret) {
            //annualized return -> average return per year 
            let annualized_return = avr * TRADING_DAYS_PER_YEAR as f64;
            let annualized_volatility = std_dev * (TRADING_DAYS_PER_YEAR as f64).sqrt();
            //sharpe ratio
            if annualized_volatility > 0.0 {
                let sharpe = (annualized_return - rf) / annualized_volatility;
                sim_sharpe_r.push(sharpe);
            }
        }
    }
    sim_sharpe_r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Candle;
    use chrono::NaiveDate;

    // Helper function to create a Candle easily
    fn candle(close: f64) -> Candle {
        Candle {
            date: NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
            open: close,
            high: close,
            low: close,
            close,
            volume: 100.0,
        }
    }

    // -------------------
    // Tests for daily_returns
    // -------------------

    #[test]
    fn test_daily_returns_empty() {
        let candles: Vec<Candle> = vec![];
        let returns = daily_returns(&candles);
        assert!(returns.is_empty());
    }

    #[test]
    fn test_daily_returns_one_candle() {
        let candles = vec![candle(100.0)];
        let returns = daily_returns(&candles);
        assert!(returns.is_empty());
    }

   #[test]
fn test_daily_returns_two_candles() {
    let candles: Vec<Candle> = vec![candle(100.0), candle(110.0)];
    let returns: Vec<f64> = daily_returns(&candles);
    // Add the f64 suffix to the literals
    let expected = (110.0_f64 / 100.0_f64).ln();
    assert_eq!(returns.len(), 1);
    assert!((returns[0] - expected).abs() < 1e-10);
}
#[test]
fn test_daily_returns_multiple_candles() {
    let candles: Vec<Candle> = vec![candle(100.0), candle(105.0), candle(110.0)];
    let returns: Vec<f64> = daily_returns(&candles);

    // Add the f64 suffix to the literals in the vector
    let expected: Vec<f64> = vec![
        (105.0_f64 / 100.0_f64).ln(),
        (110.0_f64 / 105.0_f64).ln(),
    ];

    assert_eq!(returns.len(), expected.len());
    for (r, e) in returns.iter().zip(expected.iter()) {
        assert!((r - e).abs() < 1e-10);
    }
}
    

    // -------------------
    // Tests for calc_stats
    // -------------------

    #[test]
    fn test_calc_stats_empty() {
        let returns: Vec<f64> = vec![];
        assert!(calc_stats(&returns).is_none());
    }

    #[test]
    fn test_calc_stats_single_value() {
        let returns = vec![0.01];
        assert!(calc_stats(&returns).is_none());
    }

    #[test]
    fn test_calc_stats_known_values() {
        let returns = vec![0.0, 0.0, 0.0];
        let (mean, std) = calc_stats(&returns).unwrap();
        assert!((mean - 0.0).abs() < 1e-10);
        assert!((std - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_calc_stats_nontrivial_values() {
        let returns = vec![0.01, 0.02, 0.03];
        let (mean, std) = calc_stats(&returns).unwrap();
        let expected_mean = 0.02;
        let expected_std = ((0.01_f64.powi(2) + 0.0 + 0.01_f64.powi(2)) / 2.0).sqrt();
        assert!((mean - expected_mean).abs() < 1e-10);
        assert!((std - expected_std).abs() < 1e-10);
    }

    // -------------------
    // Tests for monte_carlo_sharpe
    // -------------------

    #[test]
    fn test_monte_carlo_sharpe_basic() {
        let avr = 0.0005;    // 0.05% daily
        let std_dev = 0.01;  // 1% daily
        let rf = 0.02;       // 2% annual
        let n_sims = 10;

        let sharpe_ratios = monte_carlo_sharpe(avr, std_dev, rf, n_sims);

        // Should produce exactly n_sims results
        assert_eq!(sharpe_ratios.len(), n_sims);

        // Some ratios should be non-zero
        assert!(sharpe_ratios.iter().any(|&s| s != 0.0));
    }

    #[test]
    fn test_monte_carlo_sharpe_zero_volatility() {
        let avr = 0.001;
        let std_dev = 0.0;
        let rf = 0.01;
        let n_sims = 5;

        let sharpe_ratios = monte_carlo_sharpe(avr, std_dev, rf, n_sims);

        // All results should be empty because volatility = 0 (division by zero avoided)
        assert!(sharpe_ratios.is_empty());
    

}
}


