use std::path::PathBuf;

use clap::Parser;
use market_backtest::{data, metrics};

/// Command line interface
#[derive(Parser, Debug)]
#[command(name = "Market Backtest")]
#[command(about = "Run a backtest on a CSV file of market data", long_about = None)]
struct Args {
    /// Path to the portfolio CSV file
    #[arg(short, long)]
    file: PathBuf,

    /// Path to the benchmark CSV file
    #[arg(short = 'b', long)]
    benchmark: PathBuf,

    /// Annual risk-free rate (used if no CSV provided)
    #[arg(short = 'r', long, default_value_t = 0.02)]
    risk_free: f64,

    /// Path to CSV file with T-bill yields (e.g. 1-month treasury)
    #[arg(short = 's', long)]
    risk_free_file: Option<PathBuf>,

    /// Column in T-bill CSV to use (e.g. "1 Mo")
    #[arg(short = 'm', long, default_value = "1 Mo")]
    risk_free_maturity: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // --- Load portfolio and benchmark data ---
    let candles = data::load_csv(&args.file)?;
    let bench = data::load_csv(&args.benchmark)?;

    let returns = metrics::daily_returns(&candles);
    let bench_returns = metrics::daily_returns(&bench);

    // --- Load risk-free rates ---
    let rf_daily: Vec<f64> = if let Some(rf_path) = &args.risk_free_file {
        let series = data::load_risk_free_series(rf_path, &args.risk_free_maturity)?;
        // Ensure length matches returns
        if series.len() < returns.len() {
            // pad with last value if needed
            let mut padded = series.clone();
            padded.resize(returns.len(), *series.last().unwrap());
            padded
        } else {
            series[..returns.len()].to_vec()
        }
    } else {
        // Fallback: convert CLI annual risk-free rate to daily
        let daily = (1.0 + args.risk_free).powf(1.0 / 252.0) - 1.0;
        vec![daily; returns.len()]
    };

    // --- Compute metrics ---
    if let Some((avr, std_dev)) = metrics::calc_stats(&returns) {
        println!("Portfolio Metrics:");
        println!("   - Avg Daily Return: {:.6}", avr);
        println!("   - Daily Volatility: {:.6}", std_dev);

        // Monte Carlo Sharpe
        let n_sims = 1000;
        let sharpe_sims = metrics::monte_carlo_sharpe(avr, std_dev, rf_daily[0] * 252.0, n_sims);
        let avg_sharpe = sharpe_sims.iter().sum::<f64>() / sharpe_sims.len() as f64;
        println!("   - Monte Carlo Avg Sharpe: {:.4}", avg_sharpe);

        // Beta & Alpha
        if let Some(b) = metrics::beta(&returns, &bench_returns) {
            println!("   - Beta vs Benchmark: {:.4}", b);

            if let Some(a) = metrics::alpha(&returns, &bench_returns, &rf_daily) {
                println!("   - Alpha vs Benchmark: {:.6}", a);
            } else {
                eprintln!("Could not calculate alpha (check lengths)");
            }
        } else {
            eprintln!("Could not calculate beta (check lengths)");
        }
    } else {
        eprintln!("Not enough return data to calculate metrics");
    }

    Ok(())
}







// use std::path::PathBuf;

// use clap::Parser;
// use market_backtest::{data, metrics};

// /// command line interface
// #[derive(Parser, Debug)]
// #[command(name = "Market Backtest")]
// #[command(about = "Run a backtest on a CSV file of market data", long_about = None)]
// struct Args {
//     /// Path to the CSV file
//     #[arg(short, long)]
//     file: PathBuf,

//     /// Path to the benchmark CSV file
//     #[arg(short = 'b', long)]
//     benchmark: PathBuf,

//     /// Annual risk-free rate (e.g., 0.02 = 2%)
//     #[arg(short = 'r', long, default_value_t = 0.02)]
//     risk_free: f64,


//     /// Path to the CSV file with daily risk-free rates (e.g., 3-month T-bill)
//     #[arg(short = 'f', long)]
//     risk_free_file: PathBuf,
// }
// fn main() {
//     let args = Args::parse();

//     let candles = data::load_csv(&args.file).expect("Failed to load data");
//     println!("Loaded {} candles", candles.len());

//     // Load benchmark data
//     let bench = data::load_csv(&args.benchmark).expect("Failed to load benchmark data");
//     println!("Loaded {} benchmark candles", bench.len());

//     for c in &candles {
//         println!("{:?}", c);
//     }
//     let returns = metrics::daily_returns(&candles);
//     // Benchmark returns
//     let bench_returns = metrics::daily_returns(&bench);
//     if let Some((avr, std_dev)) = metrics::calc_stats(&returns) {
//         println!("   - Average Daily Return: {:.6}", avr);
//         println!("   - Std Deviation (Volatility): {:.6}", std_dev);

//         //Monte Carlo Sharpe Ratio Simulations
//         let rf = 0.2; //annual risk free rate
//         let n_sims = 1000;
//         let sharpe_ratios = metrics::monte_carlo_sharpe(avr, std_dev, rf, n_sims);
//         let avg_sharpe = sharpe_ratios.iter().sum::<f64>() / sharpe_ratios.len() as f64;
//         println!("   - Monte Carlo Average SR: {:.4}", avg_sharpe);

//         //Beta
//         if let Some(beta) = metrics::beta(&returns, &bench_returns) {
//             println!("   - Beta vs Benchmark: {:.4}", beta);

//             // Alpha (needs beta + risk-free rate)
//             if let Some(alpha) = metrics::alpha(&returns, &bench_returns, rf) {
//                 println!("   - Alpha vs Benchmark: {:.6}", alpha);
//             } else {
//                 eprintln!("Could not calculate alpha");
//             }
//         } else {
//             eprintln!("Could not calculate beta");
//         }
//     } else {
//         eprintln!("Not enough data points");
//     }
// }
