//Generation of signals for ORB(open range breakout) strategy
//reading the NIFTY csv with with 1 min OHLC

NIFTY:("PFFFFJ";enlist csv)0:hsym `$"C:/Users/hbtra_btlng/python/NIFTY 50_minute_data.csv"

//converting 1 min OHLCV to 5 min candle

nifty:select open:first open,high:max high, low:min low,close:last close,volume:sum volume  by datetime:(`timespan$`minute$5) xbar date from NIFTY

//adding bulliah and bearish condition if 09:25:00 candle's close>open then bullish if close<open then bearish

nifty:update candle_type:?[close>open;`bullish;`bearish],candle_val:?[close>open;high;low] from nifty where (`time$datetime)=09:25:00

//making of function of above explanation

sign_func:{[t;v;c]$[(t~`bearish) and (c<v);-1;(t~`bullish) and (c>v);1;0]}

//generating signals if after 09:25 and before 15:15 when my close>candle_val if bullish then +1(buy) or when my close<candle_val and if bearish then -1(sell) 

nifty2:update date:`date$datetime,signal:sign_func'[candle_type;candle_val;close] from fills nifty

//enter_trade:select et:datetime@first where signal=-1 by date from nifty2 where (`time$datetime) within (09:30;15:15)

enter_trade:select et:datetime@first where signal<>0 by date from nifty2 where (`time$datetime) within (09:30;15:15)

nifty3:update entry_price:close from (nifty2 lj enter_trade) where et=datetime

exit_trade:select date, exit_time:datetime, exit_price:open from nifty2 where (`time$datetime)=15:15:00, date in (exec distinct date from nifty3 where not null entry_price);

nifty3:update entry_price:?[datetime=et;close;0n], exit_price:?[datetime=exit_time;exit_price;0n] from (nifty2 lj enter_trade) lj `date xkey exit_trade

nifty4: delete et, exit_time from nifty3

entry_exit: select entry_price:first entry_price[where not null entry_price], exit_price:first exit_price[where not null exit_price] by date from nifty4

nifty3:update entry_price:?[datetime=et;close;0n], exit_price:?[datetime=exit_time;exit_price;0n] from (nifty2 lj enter_trade) lj `date xkey exit_trade;

nifty4: delete et, exit_time from nifty3;

trades: select entry_price:max entry_price, exit_price:max exit_price, signal:first signal where not null entry_price by date from nifty4 where not null entry_price or not null exit_price;

trades: delete from trades where signal=0N;

trades: update gross_pnl:?[signal=-1;entry_price-exit_price;exit_price-entry_price] from trades;

trades: update net_pnl:gross_pnl - (exit_price-entry_price)*0.0012 from trades;

trades: update cum_pnl:sums net_pnl from trades;

trades: update running_max:maxs cum_pnl from trades;

trades: update drawdown:cum_pnl - running_max from trades;

total_pnl:sum exec net_pnl from trades;

max_dd: min exec drawdown from trades;

sharpe: (sum exec net_pnl from trades) % dev exec net_pnl from trades;

calmar: (sum exec net_pnl from trades) % abs max_dd;

\ts {
NIFTY:("PFFFFJ";enlist csv)0:hsym `$"C:/Users/hbtra_btlng/python/NIFTY 50_minute_data.csv";
nifty:select open:first open,high:max high, low:min low,close:last close,volume:sum volume  by datetime:(`timespan$`minute$5) xbar date from NIFTY;
nifty:update candle_type:?[close>open;`bullish;`bearish],candle_val:?[close>open;high;low] from nifty where (`time$datetime)=09:25:00;
sign_func:{[t;v;c]$[(t~`bearish) and (c<v);-1;(t~`bullish) and (c>v);1;0]};
nifty2:update date:`date$datetime,signal:sign_func'[candle_type;candle_val;close] from fills nifty;
enter_trade:select et:datetime@first where signal<>0 by date from nifty2 where (`time$datetime) within (09:30;15:15);
nifty3:update entry_price:close from (nifty2 lj enter_trade) where et=datetime;
exit_trade:select date, exit_time:datetime, exit_price:open from nifty2 where (`time$datetime)=15:15:00, date in (exec distinct date from nifty3 where not null entry_price);
nifty3:update entry_price:?[datetime=et;close;0n], exit_price:?[datetime=exit_time;exit_price;0n] from (nifty2 lj enter_trade) lj `date xkey exit_trade;
nifty4: delete et, exit_time from nifty3;
trades: select entry_price:max entry_price, exit_price:max exit_price, signal:first signal where not null entry_price by date from nifty4 where not null entry_price or not null entry_price;
trades: delete from trades where signal=0N;
trades: update gross_pnl:?[signal=-1;entry_price-exit_price;exit_price-entry_price] from trades;
trades: update net_pnl:gross_pnl - (exit_price-entry_price)*0.0012 from trades;
trades: update cum_pnl:sums net_pnl from trades;
trades: update running_max:maxs cum_pnl from trades;
trades: update drawdown:cum_pnl - running_max from trades;
total_pnl:sum exec net_pnl from trades;
max_dd: min exec drawdown from trades;
sharpe: (sum exec net_pnl from trades) % dev exec net_pnl from trades;
calmar: (avg exec net_pnl from trades) % abs max_dd;
}

