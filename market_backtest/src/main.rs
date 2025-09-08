use market_backtest::{data, metrics};

fn main() {
    let candles = data::load_csv("data/SPDR_etf.csv").expect("Failed to load data");
    println!("Loaded {} candles", candles.len());

    for c in &candles {
        println!("{:?}", c);
    }
    let returns = metrics::daily_returns(&candles);
    if let Some((avr, std_dev)) = metrics::calc_stats(&returns) {
        println!("   - Average Daily Return: {:.6}", avr);
        println!("   - Std Deviation (Volatility): {:.6}", std_dev);

        //Monte Carlo Sharpe Ratio Simulations
        let rf = 0.2; //annual risk free rate
        let n_sims = 1000;
        let sharpe_ratios = metrics::monte_carlo_sharpe(avr, std_dev, rf, n_sims);

        let avg_sharpe = sharpe_ratios.iter().sum::<f64>() / sharpe_ratios.len() as f64;
        println!("   - Monte Carlo Average SR: {:.4}", avg_sharpe);
    } else {
        eprintln!("Not enough data points");
    }
}
