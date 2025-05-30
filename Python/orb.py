import pandas as pd
import numpy as np
from datetime import time
import warnings
import time as timer
warnings.filterwarnings('ignore')

def load_and_prepare_data(csv_path):
    """Load NIFTY data and prepare 5-minute OHLCV bars"""
    # Load data
    df = pd.read_csv(csv_path)
    
    # Convert datetime column (adjust column name as needed)
    datetime_col = df.columns[0]  # Assuming first column is datetime
    df['datetime'] = pd.to_datetime(df[datetime_col])
    df = df.set_index('datetime')
    
    # Create 5-minute OHLCV bars
    nifty = df.resample('5min').agg({
        'open': 'first',
        'high': 'max', 
        'low': 'min',
        'close': 'last',
        'volume': 'sum'
    }).dropna()
    
    nifty = nifty.reset_index()
    nifty['date'] = nifty['datetime'].dt.date
    nifty['time'] = nifty['datetime'].dt.time
    
    return nifty

def identify_signal_candles(nifty):
    """Identify 9:25 AM candles and generate signals"""
    # Filter 9:25 AM candles
    signal_candles = nifty[nifty['time'] == time(9, 25)].copy()
    
    if signal_candles.empty:
        return nifty
    
    # Determine candle type and key level
    signal_candles['candle_type'] = np.where(
        signal_candles['close'] > signal_candles['open'], 
        'bullish', 
        'bearish'
    )
    
    signal_candles['candle_val'] = np.where(
        signal_candles['candle_type'] == 'bullish',
        signal_candles['high'],
        signal_candles['low']
    )
    
    # Merge signal info back to main dataframe
    signal_info = signal_candles[['date', 'candle_type', 'candle_val']].set_index('date')
    nifty_with_signals = nifty.set_index('date').join(signal_info, how='left')
    nifty_with_signals = nifty_with_signals.reset_index()
    
    # Forward fill signal information for each day
    nifty_with_signals[['candle_type', 'candle_val']] = nifty_with_signals.groupby('date')[['candle_type', 'candle_val']].ffill()
    
    return nifty_with_signals

def generate_trading_signals(nifty_with_signals):
    """Generate buy/sell signals based on breakouts"""
    def signal_func(row):
        if pd.isna(row['candle_type']) or pd.isna(row['candle_val']):
            return 0
        
        candle_type = row['candle_type']
        candle_val = row['candle_val']
        close_price = row['close']
        
        # Bearish candle: short when price breaks below low
        if candle_type == 'bearish' and close_price < candle_val:
            return -1
        # Bullish candle: long when price breaks above high  
        elif candle_type == 'bullish' and close_price > candle_val:
            return 1
        else:
            return 0
    
    # Apply signal function
    nifty_with_signals['signal'] = nifty_with_signals.apply(signal_func, axis=1)
    
    return nifty_with_signals

def identify_trades(df):
    """Identify entry and exit points for trades"""
    trading_hours = (df['time'] >= time(9, 30)) & (df['time'] <= time(15, 15))
    trading_df = df[trading_hours].copy()
    
    trades_list = []
    
    for date in trading_df['date'].unique():
        day_data = trading_df[trading_df['date'] == date].copy()
        
        # Find first signal of the day
        signal_rows = day_data[day_data['signal'] != 0]
        
        if len(signal_rows) == 0:
            continue
            
        # Get first signal
        first_signal = signal_rows.iloc[0]
        entry_time = first_signal['datetime']
        entry_price = first_signal['close']
        signal = first_signal['signal']
        
        # Find exit at 15:15 (or closest available time)
        exit_candidates = day_data[day_data['time'] == time(15, 15)]
        
        if len(exit_candidates) == 0:
            # If no exact 15:15, take last available price
            exit_row = day_data.iloc[-1]
        else:
            exit_row = exit_candidates.iloc[0]
            
        exit_time = exit_row['datetime']
        exit_price = exit_row['open']  # Use open price for exit as in original
        
        trades_list.append({
            'date': date,
            'entry_time': entry_time,
            'entry_price': entry_price,
            'exit_time': exit_time, 
            'exit_price': exit_price,
            'signal': signal
        })
    
    return pd.DataFrame(trades_list)

def calculate_performance_metrics(trades):
    """Calculate PnL and performance metrics"""
    if trades.empty:
        return trades, 0, 0, 0, 0
    
    # Calculate gross PnL
    trades['gross_pnl'] = np.where(
        trades['signal'] == -1,
        trades['entry_price'] - trades['exit_price'],  # Short position
        trades['exit_price'] - trades['entry_price']   # Long position
    )
    
    # Calculate net PnL (including transaction costs)
    price_diff = np.abs(trades['exit_price'] - trades['entry_price'])
    trades['transaction_cost'] = price_diff * 0.0012
    trades['net_pnl'] = trades['gross_pnl'] - trades['transaction_cost']
    
    # Calculate cumulative metrics
    trades['cum_pnl'] = trades['net_pnl'].cumsum()
    trades['running_max'] = trades['cum_pnl'].cummax()
    trades['drawdown'] = trades['cum_pnl'] - trades['running_max']
    
    # Performance metrics
    total_pnl = trades['net_pnl'].sum()
    max_drawdown = trades['drawdown'].min()
    
    # Sharpe ratio (assuming daily returns)
    sharpe_ratio = trades['net_pnl'].mean() / trades['net_pnl'].std() if trades['net_pnl'].std() != 0 else 0
    
    # Calmar ratio
    avg_daily_pnl = trades['net_pnl'].mean()
    calmar_ratio = avg_daily_pnl / abs(max_drawdown) if max_drawdown != 0 else 0
    
    return trades, total_pnl, max_drawdown, sharpe_ratio, calmar_ratio

def run_nifty_strategy(csv_path):
    """Main function to run the complete trading strategy"""
    start_time = timer.time()
    
    print("Loading and preparing data...")
    step_start = timer.time()
    nifty = load_and_prepare_data(csv_path)
    print(f"Data loading completed in {timer.time() - step_start:.2f} seconds")
    
    print("Identifying signal candles...")
    step_start = timer.time()
    nifty_with_signals = identify_signal_candles(nifty)
    print(f"Signal identification completed in {timer.time() - step_start:.2f} seconds")
    
    print("Generating trading signals...")
    step_start = timer.time()
    nifty_final = generate_trading_signals(nifty_with_signals)
    print(f"Signal generation completed in {timer.time() - step_start:.2f} seconds")
    
    print("Identifying trades...")
    step_start = timer.time()
    trades = identify_trades(nifty_final)
    print(f"Trade identification completed in {timer.time() - step_start:.2f} seconds")
    
    print("Calculating performance metrics...")
    step_start = timer.time()
    trades_final, total_pnl, max_dd, sharpe, calmar = calculate_performance_metrics(trades)
    print(f"Performance calculation completed in {timer.time() - step_start:.2f} seconds")
    
    total_time = timer.time() - start_time
    
    # Print results
    print(f"\n{'='*50}")
    print("TRADING STRATEGY RESULTS")
    print(f"{'='*50}")
    print(f"Total Execution Time: {total_time:.2f} seconds")
    print(f"Total Trades: {len(trades_final)}")
    print(f"Total PnL: {total_pnl:.2f}")
    print(f"Max Drawdown: {max_dd:.2f}")
    print(f"Sharpe Ratio: {sharpe:.4f}")
    print(f"Calmar Ratio: {calmar:.4f}")
    
    # Additional statistics
    if len(trades_final) > 0:
        winning_trades = len(trades_final[trades_final['net_pnl'] > 0])
        win_rate = winning_trades / len(trades_final) * 100
        avg_win = trades_final[trades_final['net_pnl'] > 0]['net_pnl'].mean() if winning_trades > 0 else 0
        avg_loss = trades_final[trades_final['net_pnl'] < 0]['net_pnl'].mean() if len(trades_final) - winning_trades > 0 else 0
        
        print(f"Win Rate: {win_rate:.1f}%")
        print(f"Average Win: {avg_win:.2f}")
        print(f"Average Loss: {avg_loss:.2f}")
        
    return trades_final, nifty_final

# Example usage:
if __name__ == "__main__":
    # Update this path to your CSV file
    csv_path = "C:/Users/hbtra_btlng/python/NIFTY 50_minute_data.csv"
    
    try:
        trades_df, data_df = run_nifty_strategy(csv_path)
        
        # Display first few trades
        print(f"\nFirst 5 Trades:")
        print(trades_df.head())
        
        # Save results
        trades_df.to_csv("nifty_trades_results.csv", index=False)
        print(f"\nTrades saved to: nifty_trades_results.csv")
        
    except FileNotFoundError:
        print(f"File not found: {csv_path}")
        print("Please update the csv_path variable with the correct file path.")
    except Exception as e:
        print(f"Error: {e}")
        print("Please check your data format and file path.")