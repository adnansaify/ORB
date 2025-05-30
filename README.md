# Opening Range Breakout (ORB) Trading Strategy
*Multi-Language Performance Comparison & Analysis*

## 🎯 Overview

This repository demonstrates the implementation of an Opening Range Breakout (ORB) trading strategy across three different programming languages, showcasing performance optimization and domain expertise in quantitative finance. The strategy is based on the research paper ["Opening Range Breakout Strategy"](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4729284) and analyzes NIFTY 50 minute-level data to identify breakout patterns from the 9:25 AM opening range candle.

## 📈 Strategy Logic

The ORB strategy follows these key principles:

1. **Signal Generation**: Analyze the 9:25 AM candle to determine market bias
   - **Bullish Signal**: When price breaks above the high of a bullish 9:25 AM candle
   - **Bearish Signal**: When price breaks below the low of a bearish 9:25 AM candle

2. **Trade Execution**: 
   - Entry: First signal between 9:30 AM - 3:15 PM
   - Exit: 3:15 PM (end of trading session)
   - Transaction cost: 0.12% of trade value

3. **Risk Management**: Single trade per day with defined entry/exit rules

## 🚀 Implementation Languages

| Language | Execution Time | Memory Efficiency | Code Complexity |
|----------|---------------|-------------------|-----------------|
| **KDB+/Q** | ~0.001s | Highest | Lowest |
| **Rust** | 6.69s | High | Medium |
| **Python** | 25.38s | Medium | Highest |

### Performance Analysis

- **KDB+/Q**: Columnar database operations provide near-instantaneous execution for time-series analysis
- **Rust**: Memory-safe systems programming with zero-cost abstractions delivers excellent performance
- **Python**: Pandas-based implementation offers readability and rapid prototyping capabilities

## 📊 Results Summary

All implementations produce identical trading results:
- **Total Trades**: 2,140
- **Total P&L**: ₹2,051.04
- **Win Rate**: 51.2%
- **Max Drawdown**: ₹-4,714.41
- **Sharpe Ratio**: 0.0097

## 🛠️ Project Structure

```
orb-strategy/
├── README.md
├── LICENSE
├── .gitignore
├── rust/
│   ├── src/
│   │   └── main.rs
│   ├── Cargo.toml
│   └── execution_log.txt
├── python/
│   ├── orb.py
│   └── execution_log.txt
├── kdb/
│   ├── ORB2.q
│   └── performance_notes.md
└── results/
    └── nifty_trades_results.csv
```

## 🔧 Getting Started

### Prerequisites
- **Rust**: Install via [rustup](https://rustup.rs/)
- **Python**: 3.8+ with pandas, numpy
- **KDB+/Q**: Personal edition available from [KX Systems](https://kx.com/)

### Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/adnansaify/ORB/.git
   cd orb-strategy
   ```

2. **Run Rust Implementation**
   ```bash
   cd rust
   cargo run --release
   ```

3. **Run Python Implementation**
   ```bash
   cd python
   pip install pandas numpy
   python orb.py
   ```

4. **Run KDB+/Q Implementation**
   ```bash
   cd kdb
   q ORB2.q
   ```

## 📈 Data Requirements

The strategy uses NIFTY 50 minute-level OHLCV data. You can download the dataset from:
**[NIFTY 50 Minute Data on Kaggle](https://www.kaggle.com/datasets/debashis74017/nifty-50-minute-data)**

Expected CSV format:
```csv
datetime,open,high,low,close,volume
2015-01-09 09:15:00,8283.45,8284.00,8280.00,8283.45,1000
```

Update the file path in each implementation to point to your downloaded data file.

## 🎯 Key Features

- **Cross-Language Validation**: Identical results across all implementations
- **Performance Benchmarking**: Execution time comparison and analysis  
- **Production-Ready Code**: Error handling, logging, and comprehensive testing
- **Modular Design**: Easy to extend and modify strategy parameters
- **Data Pipeline**: Efficient OHLCV aggregation from minute to 5-minute bars

## 📋 Technical Highlights

### Rust Implementation
- Zero-cost abstractions with excellent memory management
- Comprehensive error handling with `anyhow` crate
- Efficient CSV parsing and datetime operations
- Modular struct-based architecture

### Python Implementation  
- Pandas-optimized vectorized operations
- Clean, readable algorithm implementation
- Comprehensive performance metrics calculation
- Professional logging and timing

### KDB+/Q Implementation
- Columnar database operations for time-series
- Functional programming paradigm
- Ultra-fast execution on large datasets
- Concise, domain-specific language syntax

## 🔍 Use Cases

This repository demonstrates:
- **Algorithmic Trading Strategy Development**
- **Multi-Language Performance Optimization**  
- **Financial Time-Series Analysis**
- **Cross-Platform System Architecture**
- **Quantitative Research Methodology**

## 📝 Contributing

Contributions welcome! Please read our contributing guidelines and submit pull requests for:
- Strategy enhancements
- Additional language implementations
- Performance optimizations
- Documentation improvements

## 📄 References

- **Strategy Research**: [Opening Range Breakout Strategy in Indian Market](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4729284)
- **Data Source**: [NIFTY 50 Minute Data on Kaggle](https://www.kaggle.com/datasets/debashis74017/nifty-50-minute-data)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This implementation is for educational and research purposes only. Past performance does not guarantee future results. Always conduct thorough backtesting and risk assessment before deploying any trading strategy.

---

*Built with ❤️ for the quantitative finance community*
