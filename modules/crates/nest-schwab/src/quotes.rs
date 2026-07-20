//! Typed models for the `/quotes` endpoint.
//!
//! Modeled directly from a real sample response covering every
//! `assetMainType` Schwab returned in it (`EQUITY`, `MUTUAL_FUND`, `INDEX`,
//! `OPTION`, `FUTURE`, `FOREX`) — not guessed. The `quote`/`reference`/
//! `fundamental` object shapes vary a lot between asset types (options
//! carry greeks, futures carry expiration/settlement fields, indexes carry
//! neither `regular` nor `fundamental` at all), so most fields are
//! `Option<T>`: a field being `Some` or `None` reflects which asset type
//! the entry actually is, not an unverified guess. `description`/
//! `exchange`/`exchangeName` were present on every asset type in the
//! sample, so those stay required.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A `/quotes` response: symbol → [`Quote`].
pub type QuotesResponse = HashMap<String, Quote>;

/// High-level instrument category, as reported by Schwab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetMainType {
    /// Common stock.
    #[serde(rename = "EQUITY")]
    Equity,
    /// Mutual fund.
    #[serde(rename = "MUTUAL_FUND")]
    MutualFund,
    /// Market index (e.g. `$SPX`, `$DJI`).
    #[serde(rename = "INDEX")]
    Index,
    /// Option contract.
    #[serde(rename = "OPTION")]
    OptionContract,
    /// Futures contract.
    #[serde(rename = "FUTURE")]
    Future,
    /// Foreign exchange pair.
    #[serde(rename = "FOREX")]
    Forex,
    /// An asset type not yet seen in a real sample response.
    #[serde(other)]
    Unknown,
}

/// A single symbol's quote entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// High-level instrument category.
    #[serde(rename = "assetMainType")]
    pub asset_main_type: AssetMainType,
    /// Finer-grained category (e.g. `"ETF"` for equity-type entries).
    #[serde(rename = "assetSubType")]
    pub asset_sub_type: Option<String>,
    /// The symbol this entry is for.
    pub symbol: String,
    /// Quote type (e.g. `"NBBO"`), present for equity-type entries.
    #[serde(rename = "quoteType")]
    pub quote_type: Option<String>,
    /// Whether this is a real-time (vs delayed) quote.
    pub realtime: bool,
    /// Schwab security identifier. `-1` or `0` for some non-equity types.
    pub ssid: i64,
    /// Reference/descriptive data for the instrument.
    pub reference: QuoteReference,
    /// Pricing data. Present for every asset type seen so far.
    pub quote: Option<QuoteDetail>,
    /// Regular-trading-hours-only pricing. Absent for indexes, futures,
    /// forex, and mutual funds in the sample.
    pub regular: Option<QuoteRegular>,
    /// Fundamentals (dividends, EPS, P/E). Absent for indexes, options,
    /// futures, and forex in the sample.
    pub fundamental: Option<QuoteFundamental>,
}

/// Reference/descriptive data — field presence varies by [`AssetMainType`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteReference {
    /// Human-readable instrument description.
    pub description: String,
    /// Exchange code.
    pub exchange: String,
    /// Exchange display name.
    #[serde(rename = "exchangeName")]
    pub exchange_name: String,
    /// CUSIP. Absent for indexes, options, futures, and forex.
    pub cusip: Option<String>,
    /// OTC market tier (e.g. `"PC"`, `"EM"`, `"QX"`), for OTC-traded equities.
    #[serde(rename = "otcMarketTier")]
    pub otc_market_tier: Option<String>,
    /// `"C"` or `"P"`. Option contracts only.
    #[serde(rename = "contractType")]
    pub contract_type: Option<String>,
    /// Option contracts only.
    #[serde(rename = "daysToExpiration")]
    pub days_to_expiration: Option<i64>,
    /// Option contracts only.
    #[serde(rename = "expirationDay")]
    pub expiration_day: Option<i64>,
    /// Option contracts only.
    #[serde(rename = "expirationMonth")]
    pub expiration_month: Option<i64>,
    /// Option contracts only.
    #[serde(rename = "expirationYear")]
    pub expiration_year: Option<i64>,
    /// Option contracts only.
    #[serde(rename = "isPennyPilot")]
    pub is_penny_pilot: Option<bool>,
    /// Option contracts only. Unix epoch milliseconds.
    #[serde(rename = "lastTradingDay")]
    pub last_trading_day: Option<i64>,
    /// Contract multiplier. Option and futures contracts.
    pub multiplier: Option<f64>,
    /// Option contracts only.
    #[serde(rename = "settlementType")]
    pub settlement_type: Option<String>,
    /// Option contracts only.
    #[serde(rename = "strikePrice")]
    pub strike_price: Option<f64>,
    /// Underlying symbol. Option contracts only.
    pub underlying: Option<String>,
    /// Option contracts only.
    #[serde(rename = "uvExpirationType")]
    pub uv_expiration_type: Option<String>,
    /// Futures contracts only.
    #[serde(rename = "futureActiveSymbol")]
    pub future_active_symbol: Option<String>,
    /// Futures contracts only. Unix epoch milliseconds.
    #[serde(rename = "futureExpirationDate")]
    pub future_expiration_date: Option<i64>,
    /// Futures contracts only.
    #[serde(rename = "futureIsActive")]
    pub future_is_active: Option<bool>,
    /// Futures contracts only.
    #[serde(rename = "futureIsTradable")]
    pub future_is_tradable: Option<bool>,
    /// Futures contracts only.
    #[serde(rename = "futureMultiplier")]
    pub future_multiplier: Option<f64>,
    /// Futures contracts only.
    #[serde(rename = "futurePriceFormat")]
    pub future_price_format: Option<String>,
    /// Futures contracts only.
    #[serde(rename = "futureSettlementPrice")]
    pub future_settlement_price: Option<f64>,
    /// Futures contracts only.
    #[serde(rename = "futureTradingHours")]
    pub future_trading_hours: Option<String>,
    /// Product code. Futures and forex.
    pub product: Option<String>,
    /// Forex only.
    #[serde(rename = "isTradable")]
    pub is_tradable: Option<bool>,
    /// Forex only.
    #[serde(rename = "marketMaker")]
    pub market_maker: Option<String>,
    /// Forex only.
    #[serde(rename = "tradingHours")]
    pub trading_hours: Option<String>,
}

/// Pricing data — field presence varies by [`AssetMainType`] (e.g. only
/// options carry greeks; only mutual funds carry `nav`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteDetail {
    /// 52-week high.
    #[serde(rename = "52WeekHigh")]
    pub week52_high: Option<f64>,
    /// 52-week low.
    #[serde(rename = "52WeekLow")]
    pub week52_low: Option<f64>,
    /// Market identifier code for the current ask.
    #[serde(rename = "askMICId")]
    pub ask_mic_id: Option<String>,
    /// Current ask price.
    #[serde(rename = "askPrice")]
    pub ask_price: Option<f64>,
    /// Current ask size.
    #[serde(rename = "askSize")]
    pub ask_size: Option<i64>,
    /// Unix epoch milliseconds of the current ask.
    #[serde(rename = "askTime")]
    pub ask_time: Option<i64>,
    /// Market identifier code for the current bid.
    #[serde(rename = "bidMICId")]
    pub bid_mic_id: Option<String>,
    /// Current bid price.
    #[serde(rename = "bidPrice")]
    pub bid_price: Option<f64>,
    /// Current bid size.
    #[serde(rename = "bidSize")]
    pub bid_size: Option<i64>,
    /// Unix epoch milliseconds of the current bid.
    #[serde(rename = "bidTime")]
    pub bid_time: Option<i64>,
    /// Previous session's close price.
    #[serde(rename = "closePrice")]
    pub close_price: Option<f64>,
    /// Session high.
    #[serde(rename = "highPrice")]
    pub high_price: Option<f64>,
    /// Market identifier code for the last trade.
    #[serde(rename = "lastMICId")]
    pub last_mic_id: Option<String>,
    /// Last trade price.
    #[serde(rename = "lastPrice")]
    pub last_price: Option<f64>,
    /// Last trade size.
    #[serde(rename = "lastSize")]
    pub last_size: Option<i64>,
    /// Session low.
    #[serde(rename = "lowPrice")]
    pub low_price: Option<f64>,
    /// Mark price.
    pub mark: Option<f64>,
    /// Change from previous close, based on mark.
    #[serde(rename = "markChange")]
    pub mark_change: Option<f64>,
    /// Percent change from previous close, based on mark.
    #[serde(rename = "markPercentChange")]
    pub mark_percent_change: Option<f64>,
    /// Net change from previous close.
    #[serde(rename = "netChange")]
    pub net_change: Option<f64>,
    /// Net percent change from previous close.
    #[serde(rename = "netPercentChange")]
    pub net_percent_change: Option<f64>,
    /// Session open price.
    #[serde(rename = "openPrice")]
    pub open_price: Option<f64>,
    /// Unix epoch milliseconds this quote was generated.
    #[serde(rename = "quoteTime")]
    pub quote_time: Option<i64>,
    /// Trading status (e.g. `"Normal"`, `"Unknown"`).
    #[serde(rename = "securityStatus")]
    pub security_status: Option<String>,
    /// Total session volume.
    #[serde(rename = "totalVolume")]
    pub total_volume: Option<i64>,
    /// Unix epoch milliseconds of the last trade.
    #[serde(rename = "tradeTime")]
    pub trade_time: Option<i64>,
    /// Implied volatility (equities: historical volatility percent).
    pub volatility: Option<f64>,
    /// Net asset value. Mutual funds only.
    #[serde(rename = "nAV")]
    pub nav: Option<f64>,
    /// Delta. Options only.
    pub delta: Option<f64>,
    /// Gamma. Options only.
    pub gamma: Option<f64>,
    /// Implied yield. Options only.
    #[serde(rename = "impliedYield")]
    pub implied_yield: Option<f64>,
    /// Indicative ask price. Options only.
    #[serde(rename = "indAskPrice")]
    pub ind_ask_price: Option<f64>,
    /// Indicative bid price. Options only.
    #[serde(rename = "indBidPrice")]
    pub ind_bid_price: Option<f64>,
    /// Unix epoch milliseconds of the indicative quote. Options only.
    #[serde(rename = "indQuoteTime")]
    pub ind_quote_time: Option<i64>,
    /// In-the-money intrinsic value. Options only.
    #[serde(rename = "moneyIntrinsicValue")]
    pub money_intrinsic_value: Option<f64>,
    /// Open interest. Options and futures.
    #[serde(rename = "openInterest")]
    pub open_interest: Option<i64>,
    /// Rho. Options only.
    pub rho: Option<f64>,
    /// Theoretical option value.
    #[serde(rename = "theoreticalOptionValue")]
    pub theoretical_option_value: Option<f64>,
    /// Theta. Options only.
    pub theta: Option<f64>,
    /// Time value. Options only.
    #[serde(rename = "timeValue")]
    pub time_value: Option<f64>,
    /// Underlying instrument's price. Options only.
    #[serde(rename = "underlyingPrice")]
    pub underlying_price: Option<f64>,
    /// Vega. Options only.
    pub vega: Option<f64>,
    /// Percent change, futures' own field distinct from `netPercentChange`.
    #[serde(rename = "futurePercentChange")]
    pub future_percent_change: Option<f64>,
    /// Unix epoch milliseconds of settlement. Futures only.
    #[serde(rename = "settleTime")]
    pub settle_time: Option<i64>,
    /// Minimum price movement. Futures and forex.
    pub tick: Option<f64>,
    /// Dollar value of one tick. Futures and forex.
    #[serde(rename = "tickAmount")]
    pub tick_amount: Option<f64>,
}

/// Regular-trading-hours-only pricing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRegular {
    /// Last regular-hours trade price.
    #[serde(rename = "regularMarketLastPrice")]
    pub regular_market_last_price: Option<f64>,
    /// Last regular-hours trade size.
    #[serde(rename = "regularMarketLastSize")]
    pub regular_market_last_size: Option<i64>,
    /// Net change during regular hours.
    #[serde(rename = "regularMarketNetChange")]
    pub regular_market_net_change: Option<f64>,
    /// Percent change during regular hours. Not present on every entry.
    #[serde(rename = "regularMarketPercentChange")]
    pub regular_market_percent_change: Option<f64>,
    /// Unix epoch milliseconds of the last regular-hours trade.
    #[serde(rename = "regularMarketTradeTime")]
    pub regular_market_trade_time: Option<i64>,
}

/// Fundamentals (dividends, EPS, P/E).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteFundamental {
    /// 10-day average volume.
    #[serde(rename = "avg10DaysVolume")]
    pub avg10_days_volume: Option<f64>,
    /// 1-year average volume.
    #[serde(rename = "avg1YearVolume")]
    pub avg1_year_volume: Option<f64>,
    /// Dividend declaration date (ISO 8601).
    #[serde(rename = "declarationDate")]
    pub declaration_date: Option<String>,
    /// Dividend amount.
    #[serde(rename = "divAmount")]
    pub div_amount: Option<f64>,
    /// Dividend ex-date (ISO 8601).
    #[serde(rename = "divExDate")]
    pub div_ex_date: Option<String>,
    /// Dividend frequency (payments per year).
    #[serde(rename = "divFreq")]
    pub div_freq: Option<i64>,
    /// Dividend pay amount.
    #[serde(rename = "divPayAmount")]
    pub div_pay_amount: Option<f64>,
    /// Dividend pay date (ISO 8601).
    #[serde(rename = "divPayDate")]
    pub div_pay_date: Option<String>,
    /// Dividend yield, percent.
    #[serde(rename = "divYield")]
    pub div_yield: Option<f64>,
    /// Earnings per share.
    pub eps: Option<f64>,
    /// Leveraged-fund leverage factor.
    #[serde(rename = "fundLeverageFactor")]
    pub fund_leverage_factor: Option<f64>,
    /// Fund strategy code (e.g. `"A"`, `"P"`).
    #[serde(rename = "fundStrategy")]
    pub fund_strategy: Option<String>,
    /// Next dividend ex-date (ISO 8601).
    #[serde(rename = "nextDivExDate")]
    pub next_div_ex_date: Option<String>,
    /// Next dividend pay date (ISO 8601).
    #[serde(rename = "nextDivPayDate")]
    pub next_div_pay_date: Option<String>,
    /// Price/earnings ratio.
    #[serde(rename = "peRatio")]
    pub pe_ratio: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real sample response (trimmed to one entry per assetMainType seen),
    // not synthesized, so this proves the model actually matches Schwab's
    // wire format rather than just an assumption of it.
    const SAMPLE: &str = r#"{
        "AAPL": {
            "assetMainType": "EQUITY", "symbol": "AAPL", "quoteType": "NBBO",
            "realtime": true, "ssid": 1973757747,
            "reference": {"cusip": "037833100", "description": "Apple Inc", "exchange": "Q", "exchangeName": "NASDAQ"},
            "quote": {
                "52WeekHigh": 169, "52WeekLow": 1.1, "askMICId": "MEMX", "askPrice": 168.41,
                "askSize": 400, "askTime": 1644854683672, "bidMICId": "IEGX", "bidPrice": 168.4,
                "bidSize": 400, "bidTime": 1644854683633, "closePrice": 177.57, "highPrice": 169,
                "lastMICId": "XADF", "lastPrice": 168.405, "lastSize": 200, "lowPrice": 167.09,
                "mark": 168.405, "markChange": -9.164999999999992, "markPercentChange": -5.161344821760428,
                "netChange": -9.165, "netPercentChange": -5.161344821760428, "openPrice": 167.37,
                "quoteTime": 1644854683672, "securityStatus": "Normal", "totalVolume": 22361159,
                "tradeTime": 1644854683408, "volatility": 0.0347
            },
            "regular": {
                "regularMarketLastPrice": 168.405, "regularMarketLastSize": 2,
                "regularMarketNetChange": -9.165, "regularMarketPercentChange": -5.161344821760428,
                "regularMarketTradeTime": 1644854683408
            },
            "fundamental": {
                "avg10DaysVolume": 1, "avg1YearVolume": 0, "divAmount": 1.1, "divFreq": 0,
                "divPayAmount": 0, "divYield": 1.1, "eps": 0, "fundLeverageFactor": 1.1, "peRatio": 1.1
            }
        },
        "AAAIX": {
            "assetMainType": "MUTUAL_FUND", "symbol": "AAAIX", "realtime": true, "ssid": -1,
            "reference": {"cusip": "025085853", "description": "American Century Strategic Allocation: Aggressive Fund - I Class", "exchange": "3", "exchangeName": "Mutual Fund"},
            "quote": {
                "52WeekHigh": 9.24, "52WeekLow": 7.48, "closePrice": 9.12, "nAV": 0,
                "netChange": -0.03, "netPercentChange": -0.32894736842104566, "securityStatus": "Normal",
                "totalVolume": 0, "tradeTime": 0
            },
            "fundamental": {
                "avg10DaysVolume": 0, "avg1YearVolume": 0, "divAmount": 0, "divFreq": 0,
                "divPayAmount": 0, "divYield": 0.83059, "eps": 0, "fundLeverageFactor": 0, "peRatio": 0
            }
        },
        "$SPX": {
            "assetMainType": "INDEX", "symbol": "$SPX", "realtime": true, "ssid": 1819771877,
            "reference": {"description": "S&P DOW JONES INDEX            S&P 500", "exchange": "0", "exchangeName": "Index"},
            "quote": {
                "52WeekHigh": 4423.46, "52WeekLow": 4385.52, "closePrice": 4766.18, "highPrice": 4423.46,
                "lastPrice": 4396.2, "lowPrice": 4385.52, "netChange": -369.98,
                "netPercentChange": -7.762610728088331, "openPrice": 4412.61, "securityStatus": "Unknown",
                "totalVolume": 628009977, "tradeTime": 1644854683056
            }
        },
        "AMZN  220617C03170000": {
            "assetMainType": "OPTION", "symbol": "AMZN  220617C03170000", "realtime": true, "ssid": 72507798,
            "reference": {
                "contractType": "C", "daysToExpiration": 123, "description": "Amazon.com Inc 06/17/2022 $3170 Call",
                "exchange": "o", "exchangeName": "OPR", "expirationDay": 17, "expirationMonth": 6,
                "expirationYear": 2022, "isPennyPilot": true, "lastTradingDay": 1655510400000,
                "multiplier": 100, "settlementType": "P", "strikePrice": 3170, "underlying": "AMZN",
                "uvExpirationType": "S"
            },
            "quote": {
                "askPrice": 223, "askSize": 2, "askTime": 0, "bidPrice": 217.65, "bidSize": 2, "bidTime": 0,
                "closePrice": 357.75, "delta": 0.5106, "gamma": 0.0007, "highPrice": 0,
                "impliedYield": 0.042, "indAskPrice": 0, "indBidPrice": 0, "indQuoteTime": 0,
                "lastPrice": 0, "lastSize": 0, "lowPrice": 0, "mark": 220.325, "markChange": -137.425,
                "markPercentChange": -38.41369671558351, "moneyIntrinsicValue": -40.795, "netChange": 0,
                "netPercentChange": 0, "openInterest": 0, "openPrice": 0, "quoteTime": 1644854683379,
                "rho": 4.5173, "securityStatus": "Normal", "theoreticalOptionValue": 221.4,
                "theta": -0.9619, "timeValue": 220.325, "totalVolume": 0, "tradeTime": 0,
                "underlyingPrice": 3129.205, "vega": 7.1633, "volatility": 32.8918
            }
        },
        "/ESZ21": {
            "assetMainType": "FUTURE", "symbol": "/ESZ21", "realtime": true, "ssid": 0,
            "reference": {
                "description": "E-mini S&P 500 Index Futures,Dec-2021,ETH", "exchange": "@",
                "exchangeName": "XCME", "futureActiveSymbol": "/ESZ21", "futureExpirationDate": 1639717200000,
                "futureIsActive": true, "futureIsTradable": true, "futureMultiplier": 50,
                "futurePriceFormat": "D,D", "futureSettlementPrice": 4696,
                "futureTradingHours": "GLBX(de=1640;0=-17001600;1=r-17001600d-15551640;7=d-16401555)",
                "product": "/ES"
            },
            "quote": {
                "askPrice": 4694.5, "askSize": 113, "askTime": 0, "bidPrice": 4694.25, "bidSize": 57,
                "bidTime": 0, "netChange": -1.5, "closePrice": 4696, "futurePercentChange": -0.0003,
                "highPrice": 4701, "lastPrice": 4694.5, "lastSize": 3, "lowPrice": 4679.25, "mark": 0,
                "openInterest": 2328678, "openPrice": 4696.5, "quoteTime": 1637168671400,
                "securityStatus": "Unknown", "settleTime": 0, "tick": 0.25, "tickAmount": 12.5,
                "totalVolume": 550778, "tradeTime": 1637168671399
            }
        },
        "EUR/USD": {
            "assetMainType": "FOREX", "symbol": "EUR/USD", "ssid": 1, "realtime": true,
            "reference": {
                "description": "Euro/USDollar Spot", "exchange": "T", "exchangeName": "GFT",
                "isTradable": false, "marketMaker": "", "product": "", "tradingHours": ""
            },
            "quote": {
                "52WeekHigh": 1.135, "52WeekLow": 1.1331, "askPrice": 1.13456, "askSize": 1000000,
                "bidPrice": 1.13434, "bidSize": 1000000, "netChange": 0.00254, "closePrice": 1.13191,
                "highPrice": 1.135, "lastPrice": 1.13445, "lastSize": 0, "lowPrice": 1.1331,
                "mark": 1.13445, "openPrice": 1.13324, "netPercentChange": 0, "quoteTime": 1637236739892,
                "securityStatus": "Unknown", "tick": 0, "tickAmount": 0, "totalVolume": 0,
                "tradeTime": 1637236739892
            }
        }
    }"#;

    #[test]
    fn deserializes_an_equity_entry_with_all_four_sections() {
        let response: QuotesResponse = serde_json::from_str(SAMPLE).expect("deserialize");
        let aapl = response.get("AAPL").expect("AAPL entry");

        assert_eq!(aapl.asset_main_type, AssetMainType::Equity);
        assert_eq!(aapl.quote_type.as_deref(), Some("NBBO"));
        assert_eq!(aapl.reference.cusip.as_deref(), Some("037833100"));
        assert_eq!(aapl.quote.as_ref().unwrap().ask_price, Some(168.41));
        assert_eq!(
            aapl.regular.as_ref().unwrap().regular_market_last_price,
            Some(168.405)
        );
        assert_eq!(aapl.fundamental.as_ref().unwrap().pe_ratio, Some(1.1));
    }

    #[test]
    fn deserializes_a_mutual_fund_entry_with_nav() {
        let response: QuotesResponse = serde_json::from_str(SAMPLE).expect("deserialize");
        let fund = response.get("AAAIX").expect("AAAIX entry");

        assert_eq!(fund.asset_main_type, AssetMainType::MutualFund);
        assert_eq!(fund.quote.as_ref().unwrap().nav, Some(0.0));
        assert!(fund.regular.is_none());
    }

    #[test]
    fn deserializes_an_index_entry_with_no_regular_or_fundamental() {
        let response: QuotesResponse = serde_json::from_str(SAMPLE).expect("deserialize");
        let spx = response.get("$SPX").expect("$SPX entry");

        assert_eq!(spx.asset_main_type, AssetMainType::Index);
        assert!(spx.reference.cusip.is_none());
        assert!(spx.regular.is_none());
        assert!(spx.fundamental.is_none());
    }

    #[test]
    fn deserializes_an_option_entry_with_greeks() {
        let response: QuotesResponse = serde_json::from_str(SAMPLE).expect("deserialize");
        let option = response.get("AMZN  220617C03170000").expect("option entry");

        assert_eq!(option.asset_main_type, AssetMainType::OptionContract);
        assert_eq!(option.reference.strike_price, Some(3170.0));
        assert_eq!(option.reference.contract_type.as_deref(), Some("C"));
        assert_eq!(option.quote.as_ref().unwrap().delta, Some(0.5106));
        assert_eq!(option.quote.as_ref().unwrap().theta, Some(-0.9619));
    }

    #[test]
    fn deserializes_a_future_entry_with_settlement_fields() {
        let response: QuotesResponse = serde_json::from_str(SAMPLE).expect("deserialize");
        let future = response.get("/ESZ21").expect("future entry");

        assert_eq!(future.asset_main_type, AssetMainType::Future);
        assert_eq!(future.reference.future_settlement_price, Some(4696.0));
        assert_eq!(future.quote.as_ref().unwrap().tick, Some(0.25));
    }

    #[test]
    fn deserializes_a_forex_entry() {
        let response: QuotesResponse = serde_json::from_str(SAMPLE).expect("deserialize");
        let forex = response.get("EUR/USD").expect("forex entry");

        assert_eq!(forex.asset_main_type, AssetMainType::Forex);
        assert_eq!(forex.reference.is_tradable, Some(false));
        assert_eq!(forex.quote.as_ref().unwrap().ask_price, Some(1.13456));
    }

    #[test]
    fn unknown_asset_main_type_does_not_fail_deserialization() {
        let json = r#"{"XYZ": {
            "assetMainType": "BOND", "symbol": "XYZ", "realtime": true, "ssid": 1,
            "reference": {"description": "d", "exchange": "e", "exchangeName": "n"}
        }}"#;
        let response: QuotesResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(
            response.get("XYZ").unwrap().asset_main_type,
            AssetMainType::Unknown
        );
    }
}
