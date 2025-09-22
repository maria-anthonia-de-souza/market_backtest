use crate::data::Candle;
use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use statrs::statistics::Statistics;

const TRADING_DAYS_PER_YEAR: usize = 252;

//
// --------------------
// Daily Returns
// --------------------
pub fn daily_returns(candles: &[Candle]) -> Vec<f64> {
    let mut returns = Vec::new();
    if candles.len() < 2 {
        return returns;
    }

    for i in 1..candles.len() {
        let ret = (candles[i].close / candles[i - 1].close).ln();
        returns.push(ret);
    }
    returns
}

//
// --------------------
// Basic Statistics
// --------------------
pub fn calc_stats(returns: &[f64]) -> Option<(f64, f64)> {
    let count = returns.len() as f64;
    if count < 2.0 {
        return None;
    }

    let mean = returns.mean();
    let variance = returns.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (count - 1.0);
    let std_dev = variance.sqrt();

    Some((mean, std_dev))
}

//
// --------------------
// Monte Carlo Sharpe Ratio
// --------------------
// `avr` and `std_dev` are daily mean & std deviation
// `rf` is annualized risk-free rate (scalar)
// `n_sims` is number of Monte Carlo simulations
pub fn monte_carlo_sharpe(avr: f64, std_dev: f64, rf: f64, n_sims: usize) -> Vec<f64> {
    let mut sim_sharpe_r = Vec::with_capacity(n_sims);
    if std_dev == 0.0 {
        return sim_sharpe_r;
    }

    let mut rng = thread_rng();
    let ret_dist = Normal::new(avr, std_dev).unwrap();

    for _ in 0..n_sims {
        // simulate one year of daily returns
        let sim_ret: Vec<f64> = (0..TRADING_DAYS_PER_YEAR)
            .map(|_| ret_dist.sample(&mut rng))
            .collect();

        if let Some((sim_avr, sim_std)) = calc_stats(&sim_ret) {
            let annual_ret = sim_avr * TRADING_DAYS_PER_YEAR as f64;
            let annual_vol = sim_std * (TRADING_DAYS_PER_YEAR as f64).sqrt();
            if annual_vol > 0.0 {
                sim_sharpe_r.push((annual_ret - rf) / annual_vol);
            }
        }
    }
    sim_sharpe_r
}

//
// --------------------
// Beta
// --------------------
pub fn beta(asset_rets: &[f64], market_rets: &[f64]) -> Option<f64> {
    if asset_rets.len() != market_rets.len() || asset_rets.len() < 2 {
        return None;
    }

    let cov = asset_rets.covariance(market_rets);
    let var_market = market_rets.variance();
    if var_market == 0.0 {
        return None;
    }

    Some(cov / var_market)
}

//
// --------------------
// Alpha
// --------------------
// Uses daily risk-free returns series to compute excess returns
pub fn alpha(asset_rets: &[f64], market_rets: &[f64], rf_rets: &[f64]) -> Option<f64> {
    if asset_rets.len() != market_rets.len() || asset_rets.len() != rf_rets.len() {
        return None;
    }

    // excess returns
    let excess_asset: Vec<f64> = asset_rets.iter().zip(rf_rets).map(|(a, rf)| a - rf).collect();
    let excess_market: Vec<f64> = market_rets.iter().zip(rf_rets).map(|(m, rf)| m - rf).collect();

    // compute beta based on excess returns
    let beta = crate::metrics::beta(&excess_asset, &excess_market)?;
    let mean_asset = excess_asset.iter().sum::<f64>() / excess_asset.len() as f64;
    let mean_market = excess_market.iter().sum::<f64>() / excess_market.len() as f64;

    Some(mean_asset - beta * mean_market)
}




// use crate::data::Candle;
// use rand::thread_rng;
// use rand_distr::{Distribution, Normal};
// use statrs::statistics::Statistics;
// // Define the number of trading days in a year as a constant.
// const TRADING_DAYS_PER_YEAR: usize = 252;

// pub fn daily_returns(candles: &[Candle]) -> Vec<f64> {
//     let mut returns: Vec<f64> = Vec::new();
//     //candles array has less then two candles we cannot calculate the daily return
//     if candles.len() < 2 {
//         return returns;
//     }
//     //loop from item 2 to the end of the list
//     for candle in 1..candles.len() {
//         let yesterday = &candles[candle - 1];
//         let today = &candles[candle];
//         //calculate log return
//         let ret = (today.close / yesterday.close).ln();
//         returns.push(ret);
//     }
//     returns
// }
// // //average returns
// // pub fn mean(data: &[f64]) -> f64 {
// //     let count = data.len() as f64;
// //     if count == 0.0 {
// //         return 0.0;
// //     }
// //     let sum: f64 = data.iter().sum();
// //     sum / count
// // }
// // //measure volatility (how spread out the numbers in the data set are) risk
// // pub fn variance(data: &[f64]) -> f64 {
// //     let mut diffs: Vec<f64> = Vec::new();
// //     for i in data {
// //         let x = i - mean(data) as f64;
// //         diffs.push(x);
// //     }
// //     let mut square_diffs: Vec<f64> = Vec::new();
// //     for diff in diffs{
// //         let square = diff * diff;
// //          square_diffs.push(square);
// //     }

// //     let sum: f64 = square_diffs.iter().sum();
// //     let count:f64= square_diffs.len() as f64;
// //     sum/(count - 1.0)
// // }

// pub fn calc_stats(returns: &[f64]) -> Option<(f64, f64)> {
//     let count = returns.len() as f64;
//     if count < 2.0 {
//         return None;
//     }
//     let sum: f64 = returns.iter().sum();
//     //average return
//     let avr = sum / count;
//     //variance -> for each data point, subtract the mean and square result
//     //sum of squared diff
//     let variance = returns
//         .iter()
//         .map(|value| {
//             let diff = value - avr;
//             diff * diff
//         })
//         .sum::<f64>()
//         / (count - 1.0); // sum is a decimal and using n-1 to correct any errors in the small dataset
//     //std dev is the squareroot of variance
//     let std_dev = variance.sqrt();
//     //if values vals exist, return  (Option)
//     Some((avr, std_dev))
// }
// //rf = risk free return, n_sims = numb of simulations
// pub fn monte_carlo_sharpe(avr: f64, std_dev: f64, rf: f64, n_sims: usize) -> Vec<f64> {
//     // generate random returns, compute Sharpe ratios, return them

//     let mut sim_sharpe_r = Vec::with_capacity(n_sims);
//     //handle 0 volatility by just returning empty vector since dividng by 0 is not possible
//     if std_dev == 0.0 {
//         return sim_sharpe_r;
//     }
//     // Standard normal distribution to generate random daily returns
//     let mut rng = thread_rng();
//     let ret_dist = Normal::new(avr, std_dev).unwrap();

//     for _ in 0..n_sims {
//         //sim returns through this range (one year)
//         let sim_ret: Vec<f64> = (0..TRADING_DAYS_PER_YEAR)
//             .map(|_| ret_dist.sample(&mut rng))
//             .collect();

//         //Compute mean & Std dev
//         if let Some((avr, std_dev)) = calc_stats(&sim_ret) {
//             //annualized return -> average return per year
//             let annualized_return = avr * TRADING_DAYS_PER_YEAR as f64;
//             let annualized_volatility = std_dev * (TRADING_DAYS_PER_YEAR as f64).sqrt();
//             //sharpe ratio
//             if annualized_volatility > 0.0 {
//                 let sharpe = (annualized_return - rf) / annualized_volatility;
//                 sim_sharpe_r.push(sharpe);
//             }
//         }
//     }
//     sim_sharpe_r
// }

// //Beta(volatility and risk) of asset returns compared to market returns
// pub fn beta(asset_rets: &[f64], market_rets: &[f64]) -> Option<f64> {
//     //making sure both the csv
//     if asset_rets.len() != market_rets.len() || asset_rets.len() < 2 {
//         return None;
//     }
//     //covariance btween asset and market returns
//     let cov = asset_rets.covariance(market_rets);
//     //variance
//     let var_market = market_rets.variance();
//     if var_market == 0.0 {
//         return None; // so we do not divide by zero
//     }
//     //Beta= Cov(asset,market)/var(market)
//     Some(cov / var_market)
// }
// //Alpha (excess returns)
// pub fn alpha(asset_rets: &[f64], market_rets: &[f64], rf: f64) -> Option<f64> {
//     //beta
//     let b = beta(asset_rets, market_rets)?;

//     //mean
//     let avr_asset = asset_rets.mean();
//     let avr_market = market_rets.mean();

//     //Alpha = asset mean return - [Rf + Beta * (Market mean - Rf)]
//     Some(avr_asset - (rf + b * (avr_market - rf)))
// }

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
        let expected: Vec<f64> = vec![(105.0_f64 / 100.0_f64).ln(), (110.0_f64 / 105.0_f64).ln()];

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
        let avr = 0.0005; // 0.05% daily
        let std_dev = 0.01; // 1% daily
        let rf = 0.02; // 2% annual
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
#[cfg(test)]
mod beta_alpha_tests {
    use super::*;
    use statrs::statistics::Statistics;

    #[test]
    fn test_beta_basic() {
        let asset = vec![0.01, 0.02, 0.03, 0.04];
        let market = vec![0.02, 0.03, 0.04, 0.05];

        let result = beta(&asset, &market).unwrap();

        let cov = asset.as_slice().covariance(market.as_slice());
        let var_m = market.as_slice().variance();
        let expected = cov / var_m;
        assert!((result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_beta_mismatched_lengths() {
        let asset = vec![0.01, 0.02];
        let market = vec![0.01];
        assert!(beta(&asset, &market).is_none());
    }

    #[test]
    fn test_beta_zero_variance() {
        let asset = vec![0.01, 0.02, 0.03];
        let market = vec![0.05, 0.05, 0.05]; // zero variance
        assert!(beta(&asset, &market).is_none());
    }

    #[test]
    fn test_alpha_basic() {
        let asset = vec![0.01, 0.02, 0.03];
        let market = vec![0.02, 0.03, 0.04];
        let rf_rets = vec![0.01, 0.01, 0.01]; // constant per-period rf

        let a = alpha(&asset, &market, &rf_rets).unwrap();

        // manual calc
        let excess_asset: Vec<f64> = asset.iter().zip(&rf_rets).map(|(a, r)| a - r).collect();
        let excess_market: Vec<f64> = market.iter().zip(&rf_rets).map(|(m, r)| m - r).collect();

        let b = beta(&excess_asset, &excess_market).unwrap();
        let mean_asset = excess_asset.mean();
        let mean_market = excess_market.mean();
        let expected = mean_asset - b * mean_market;

        assert!((a - expected).abs() < 1e-10);
    }

    #[test]
    fn test_alpha_mismatched_lengths() {
        let asset = vec![0.01, 0.02];
        let market = vec![0.01];
        let rf_rets = vec![0.01, 0.01];
        assert!(alpha(&asset, &market, &rf_rets).is_none());
    }

    #[test]
    fn test_alpha_zero_variance_market() {
        let asset = vec![0.01, 0.02, 0.03];
        let market = vec![0.05, 0.05, 0.05]; // zero variance
        let rf_rets = vec![0.01, 0.01, 0.01];
        assert!(alpha(&asset, &market, &rf_rets).is_none());
    }

    #[test]
    fn test_beta_large_dataset() {
        let asset: Vec<f64> = (1..=1000).map(|x| x as f64 * 0.001).collect();
        let market: Vec<f64> = (1..=1000).map(|x| x as f64 * 0.0015).collect();

        let result = beta(&asset, &market).unwrap();
        let cov = asset.as_slice().covariance(&market);
        let var_m = market.as_slice().variance();
        let expected = cov / var_m;

        assert!((result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_alpha_large_dataset() {
        let asset: Vec<f64> = (1..=1000).map(|x| (x as f64).sin() * 0.01).collect();
        let market: Vec<f64> = (1..=1000).map(|x| (x as f64).cos() * 0.01).collect();
        let rf_rets: Vec<f64> = vec![0.0; 1000]; // zero rf

        let result = alpha(&asset, &market, &rf_rets).unwrap();

        let excess_asset: Vec<f64> = asset.iter().zip(&rf_rets).map(|(a, r)| a - r).collect();
        let excess_market: Vec<f64> = market.iter().zip(&rf_rets).map(|(m, r)| m - r).collect();

        let mean_asset = excess_asset.as_slice().mean();
        let mean_market = excess_market.as_slice().mean();
        let beta_val = beta(&excess_asset, &excess_market).unwrap();
        let expected = mean_asset - beta_val * mean_market;

        assert!((result - expected).abs() < 1e-10);
    }
}

