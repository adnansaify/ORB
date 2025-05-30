use csv::ReaderBuilder;
use chrono::{NaiveDate, NaiveTime, NaiveDateTime, Timelike};
use std::time::Instant;
use std::collections::HashMap;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
struct CsvRow {
    date: String,  // This will contain "2015-01-09 09:15:00" format
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, Clone)]
struct OhlcBar {
    datetime: NaiveDateTime,
    date: NaiveDate,
    time: NaiveTime,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    candle_type: Option<String>,
    candle_val: Option<f64>,
    signal: i32,
}

#[derive(Debug, Clone)]
struct Trade {
    date: NaiveDate,
    entry_time: NaiveDateTime,
    entry_price: f64,
    exit_time: NaiveDateTime,
    exit_price: f64,
    signal: i32,
    gross_pnl: f64,
    net_pnl: f64,
}

#[derive(Debug)]
struct PerformanceMetrics {
    total_pnl: f64,
    max_drawdown: f64,
    sharpe_ratio: f64,
    calmar_ratio: f64,
    win_rate: f64,
    avg_win: f64,
    avg_loss: f64,
    total_trades: usize,
}

struct NiftyStrategy {
    data: Vec<OhlcBar>,
    trades: Vec<Trade>,
}

impl NiftyStrategy {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            trades: Vec::new(),
        }
    }

    fn load_and_prepare_data(&mut self, csv_path: &str) -> Result<()> {
        let step_start = Instant::now();
        
        // Read CSV file
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(csv_path)?;
        
        let mut raw_data: Vec<CsvRow> = Vec::new();
        for result in reader.deserialize() {
            let record: CsvRow = result?;
            raw_data.push(record);
        }
        
        println!("Loaded {} rows from CSV", raw_data.len());
        
        // Parse datetime and sort data
        let mut parsed_data: Vec<OhlcBar> = raw_data
            .into_iter()
            .filter_map(|row| {
                // Parse the date column which contains datetime
                let datetime = Self::parse_datetime(&row.date)?;
                Some(OhlcBar {
                    datetime,
                    date: datetime.date(),
                    time: datetime.time(),
                    open: row.open,
                    high: row.high,
                    low: row.low,
                    close: row.close,
                    volume: row.volume,
                    candle_type: None,
                    candle_val: None,
                    signal: 0,
                })
            })
            .collect();
        
        // Sort by datetime
        parsed_data.sort_by(|a, b| a.datetime.cmp(&b.datetime));
        
        // Create 5-minute OHLCV bars
        self.data = Self::create_5min_bars(parsed_data);
        
        println!("Data loading completed in {:.2} seconds", step_start.elapsed().as_secs_f64());
        println!("Created {} 5-minute bars", self.data.len());
        Ok(())
    }

    fn parse_datetime(datetime_str: &str) -> Option<NaiveDateTime> {
        // Try common datetime formats
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%d-%m-%Y %H:%M:%S", 
            "%Y/%m/%d %H:%M:%S",
            "%d/%m/%Y %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%d-%m-%Y %H:%M",
        ];
        
        for format in &formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, format) {
                return Some(dt);
            }
        }
        None
    }

    fn create_5min_bars(data: Vec<OhlcBar>) -> Vec<OhlcBar> {
        let mut five_min_bars = Vec::new();
        let mut current_group: Vec<OhlcBar> = Vec::new();
        let mut current_5min_start: Option<NaiveDateTime> = None;
        
        for bar in data {
            // Calculate 5-minute boundary
            let bar_5min_start = Self::round_to_5min(bar.datetime);
            
            if current_5min_start.is_none() {
                current_5min_start = Some(bar_5min_start);
            }
            
            if current_5min_start == Some(bar_5min_start) {
                current_group.push(bar);
            } else {
                // Process current group and start new group
                if !current_group.is_empty() {
                    five_min_bars.push(Self::aggregate_bars(&current_group));
                }
                current_group.clear();
                current_group.push(bar);
                current_5min_start = Some(bar_5min_start);
            }
        }
        
        // Process last group
        if !current_group.is_empty() {
            five_min_bars.push(Self::aggregate_bars(&current_group));
        }
        
        five_min_bars
    }

    fn round_to_5min(datetime: NaiveDateTime) -> NaiveDateTime {
        let minute = datetime.minute();
        let rounded_minute = (minute / 5) * 5;
        datetime.with_minute(rounded_minute).unwrap().with_second(0).unwrap()
    }

    fn aggregate_bars(bars: &[OhlcBar]) -> OhlcBar {
        let first = &bars[0];
        let last = &bars[bars.len() - 1];
        
        let open = first.open;
        let close = last.close;
        let high = bars.iter().map(|b| b.high).fold(0.0, f64::max);
        let low = bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let volume = bars.iter().map(|b| b.volume).sum();
        
        OhlcBar {
            datetime: Self::round_to_5min(first.datetime),
            date: first.date,
            time: Self::round_to_5min(first.datetime).time(),
            open,
            high,
            low,
            close,
            volume,
            candle_type: None,
            candle_val: None,
            signal: 0,
        }
    }

    fn identify_signal_candles(&mut self) -> Result<()> {
        let step_start = Instant::now();
        
        let target_time = NaiveTime::from_hms_opt(9, 25, 0).unwrap();
        
        // Create a map of signal candles by date
        let mut signal_map: HashMap<NaiveDate, (String, f64)> = HashMap::new();
        
        for bar in &self.data {
            if bar.time == target_time {
                let candle_type = if bar.close > bar.open {
                    "bullish".to_string()
                } else {
                    "bearish".to_string()
                };
                
                let candle_val = if bar.close > bar.open {
                    bar.high
                } else {
                    bar.low
                };
                
                signal_map.insert(bar.date, (candle_type, candle_val));
            }
        }
        
        // Apply signal information to all bars
        for bar in &mut self.data {
            if let Some((candle_type, candle_val)) = signal_map.get(&bar.date) {
                bar.candle_type = Some(candle_type.clone());
                bar.candle_val = Some(*candle_val);
            }
        }
        
        println!("Signal identification completed in {:.2} seconds", step_start.elapsed().as_secs_f64());
        println!("Found {} signal days", signal_map.len());
        Ok(())
    }

    fn generate_trading_signals(&mut self) -> Result<()> {
        let step_start = Instant::now();
        
        for bar in &mut self.data {
            if let (Some(candle_type), Some(candle_val)) = (&bar.candle_type, bar.candle_val) {
                bar.signal = match candle_type.as_str() {
                    "bearish" if bar.close < candle_val => -1,
                    "bullish" if bar.close > candle_val => 1,
                    _ => 0,
                };
            }
        }
        
        println!("Signal generation completed in {:.2} seconds", step_start.elapsed().as_secs_f64());
        Ok(())
    }

    fn identify_trades(&mut self) -> Result<()> {
        let step_start = Instant::now();
        
        let start_time = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(15, 15, 0).unwrap();
        let exit_time = NaiveTime::from_hms_opt(15, 15, 0).unwrap();
        
        // Group data by date
        let mut date_groups: HashMap<NaiveDate, Vec<&OhlcBar>> = HashMap::new();
        for bar in &self.data {
            if bar.time >= start_time && bar.time <= end_time {
                date_groups.entry(bar.date).or_insert_with(Vec::new).push(bar);
            }
        }
        
        // Process each trading day
        for (date, day_bars) in date_groups {
            // Find first signal of the day
            let first_signal = day_bars.iter()
                .find(|bar| bar.signal != 0);
            
            if let Some(entry_bar) = first_signal {
                // Find exit bar at 15:15 or last available
                let exit_bar = day_bars.iter()
                    .find(|bar| bar.time == exit_time)
                    .or_else(|| day_bars.last())
                    .unwrap();
                
                // Calculate PnL
                let gross_pnl = if entry_bar.signal == -1 {
                    entry_bar.close - exit_bar.open // Short position
                } else {
                    exit_bar.open - entry_bar.close // Long position
                };
                
                let transaction_cost = (exit_bar.open - entry_bar.close).abs() * 0.0012;
                let net_pnl = gross_pnl - transaction_cost;
                
                let trade = Trade {
                    date,
                    entry_time: entry_bar.datetime,
                    entry_price: entry_bar.close,
                    exit_time: exit_bar.datetime,
                    exit_price: exit_bar.open,
                    signal: entry_bar.signal,
                    gross_pnl,
                    net_pnl,
                };
                
                self.trades.push(trade);
            }
        }
        
        // Sort trades by date
        self.trades.sort_by(|a, b| a.date.cmp(&b.date));
        
        println!("Trade identification completed in {:.2} seconds", step_start.elapsed().as_secs_f64());
        println!("Identified {} trades", self.trades.len());
        Ok(())
    }

    fn calculate_performance_metrics(&self) -> PerformanceMetrics {
        let step_start = Instant::now();
        
        if self.trades.is_empty() {
            return PerformanceMetrics {
                total_pnl: 0.0,
                max_drawdown: 0.0,
                sharpe_ratio: 0.0,
                calmar_ratio: 0.0,
                win_rate: 0.0,
                avg_win: 0.0,
                avg_loss: 0.0,
                total_trades: 0,
            };
        }

        let total_pnl: f64 = self.trades.iter().map(|t| t.net_pnl).sum();
        
        // Calculate cumulative PnL and drawdown
        let mut cum_pnl = 0.0_f64;
        let mut running_max = 0.0_f64;
        let mut max_drawdown = 0.0_f64;
        
        for trade in &self.trades {
            cum_pnl += trade.net_pnl;
            running_max = running_max.max(cum_pnl);
            let drawdown = cum_pnl - running_max;
            max_drawdown = max_drawdown.min(drawdown);
        }

        // Calculate statistics
        let pnl_values: Vec<f64> = self.trades.iter().map(|t| t.net_pnl).collect();
        let mean_pnl = total_pnl / self.trades.len() as f64;
        
        let variance: f64 = pnl_values.iter()
            .map(|x| (x - mean_pnl).powi(2))
            .sum::<f64>() / self.trades.len() as f64;
        let std_dev = variance.sqrt();
        
        let sharpe_ratio = if std_dev != 0.0 { mean_pnl / std_dev } else { 0.0 };
        let calmar_ratio = if max_drawdown != 0.0 { mean_pnl / max_drawdown.abs() } else { 0.0 };

        // Win rate and average win/loss
        let winning_trades: Vec<&Trade> = self.trades.iter().filter(|t| t.net_pnl > 0.0).collect();
        let losing_trades: Vec<&Trade> = self.trades.iter().filter(|t| t.net_pnl < 0.0).collect();
        
        let win_rate = (winning_trades.len() as f64 / self.trades.len() as f64) * 100.0;
        let avg_win = if !winning_trades.is_empty() {
            winning_trades.iter().map(|t| t.net_pnl).sum::<f64>() / winning_trades.len() as f64
        } else { 0.0 };
        let avg_loss = if !losing_trades.is_empty() {
            losing_trades.iter().map(|t| t.net_pnl).sum::<f64>() / losing_trades.len() as f64
        } else { 0.0 };

        println!("Performance calculation completed in {:.2} seconds", step_start.elapsed().as_secs_f64());

        PerformanceMetrics {
            total_pnl,
            max_drawdown,
            sharpe_ratio,
            calmar_ratio,
            win_rate,
            avg_win,
            avg_loss,
            total_trades: self.trades.len(),
        }
    }

    fn save_results(&self, output_path: &str) -> Result<()> {
        let mut wtr = csv::Writer::from_path(output_path)?;
        
        // Write header
        wtr.write_record(&[
            "date", "entry_time", "entry_price", "exit_time", 
            "exit_price", "signal", "gross_pnl", "net_pnl"
        ])?;
        
        // Write data
        for trade in &self.trades {
            wtr.write_record(&[
                trade.date.to_string(),
                trade.entry_time.to_string(),
                trade.entry_price.to_string(),
                trade.exit_time.to_string(),
                trade.exit_price.to_string(),
                trade.signal.to_string(),
                format!("{:.4}", trade.gross_pnl),
                format!("{:.4}", trade.net_pnl),
            ])?;
        }
        
        wtr.flush()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let total_start = Instant::now();
    
    // Update this path to your CSV file
    let csv_path = "C:/Users/hbtra_btlng/python/NIFTY 50_minute_data.csv";
    let output_path = "nifty_trades_results.csv";
    
    println!("Starting NIFTY Trading Strategy...");
    println!("Using pure Rust implementation with CSV crate");
    
    let mut strategy = NiftyStrategy::new();
    
    // Run the complete strategy pipeline
    strategy.load_and_prepare_data(csv_path)?;
    strategy.identify_signal_candles()?;
    strategy.generate_trading_signals()?;
    strategy.identify_trades()?;
    
    let metrics = strategy.calculate_performance_metrics();
    let total_time = total_start.elapsed().as_secs_f64();
    
    // Print results
    println!("\n{}", "=".repeat(50));
    println!("TRADING STRATEGY RESULTS");
    println!("{}", "=".repeat(50));
    println!("Total Execution Time: {:.2} seconds", total_time);
    println!("Total Trades: {}", metrics.total_trades);
    println!("Total PnL: {:.2}", metrics.total_pnl);
    println!("Max Drawdown: {:.2}", metrics.max_drawdown);
    println!("Sharpe Ratio: {:.4}", metrics.sharpe_ratio);
    println!("Calmar Ratio: {:.4}", metrics.calmar_ratio);
    println!("Win Rate: {:.1}%", metrics.win_rate);
    println!("Average Win: {:.2}", metrics.avg_win);
    println!("Average Loss: {:.2}", metrics.avg_loss);
    
    // Save results
    strategy.save_results(output_path)?;
    println!("\nTrades saved to: {}", output_path);
    
    // Display first few trades
    if !strategy.trades.is_empty() {
        println!("\nFirst 5 Trades:");
        for (i, trade) in strategy.trades.iter().take(5).enumerate() {
            println!("{}. Date: {}, Signal: {}, Entry: {:.2}, Exit: {:.2}, PnL: {:.2}", 
                i + 1, trade.date, trade.signal, trade.entry_price, trade.exit_price, trade.net_pnl);
        }
    }
    
    // Performance summary
    println!("\n{}", "=".repeat(50));
    println!("PERFORMANCE SUMMARY");
    println!("{}", "=".repeat(50));
    println!("⚡ Pure Rust implementation");
    println!("📊 {} data points processed", strategy.data.len());
    println!("🎯 {} trading signals generated", 
        strategy.data.iter().filter(|b| b.signal != 0).count());
    println!("💰 {} profitable trades", 
        strategy.trades.iter().filter(|t| t.net_pnl > 0.0).count());
    println!("📉 {} losing trades", 
        strategy.trades.iter().filter(|t| t.net_pnl < 0.0).count());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_initialization() {
        let strategy = NiftyStrategy::new();
        assert_eq!(strategy.trades.len(), 0);
        assert_eq!(strategy.data.len(), 0);
    }

    #[test]
    fn test_performance_metrics_empty() {
        let strategy = NiftyStrategy::new();
        let metrics = strategy.calculate_performance_metrics();
        assert_eq!(metrics.total_trades, 0);
        assert_eq!(metrics.total_pnl, 0.0);
    }

    #[test]
    fn test_datetime_parsing() {
        let test_cases = [
            "2024-01-15 09:30:00",
            "15-01-2024 09:30:00",
            "2024/01/15 09:30:00",
        ];
        
        for case in &test_cases {
            assert!(NiftyStrategy::parse_datetime(case).is_some());
        }
    }

    #[test]
    fn test_5min_rounding() {
        let datetime = NaiveDateTime::parse_from_str("2024-01-15 09:37:23", "%Y-%m-%d %H:%M:%S").unwrap();
        let rounded = NiftyStrategy::round_to_5min(datetime);
        
        assert_eq!(rounded.minute(), 35);
        assert_eq!(rounded.second(), 0);
    }
}