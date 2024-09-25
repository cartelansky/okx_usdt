use reqwest;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.okx.com/api/v5/public/instruments?instType=SPOT";
    let client = reqwest::Client::new();
    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await?;

    if !response.status().is_success() {
        println!(
            "API isteği başarısız oldu. Durum kodu: {}",
            response.status()
        );
        return Ok(());
    }

    let text = response.text().await?;
    let data: Value = serde_json::from_str(&text)?;

    let mut coins: Vec<String> = Vec::new();
    if let Some(instruments) = data["data"].as_array() {
        for instrument in instruments {
            if let Some(instrument_id) = instrument["instId"].as_str() {
                if instrument_id.ends_with("-USDT") {
                    let base_ccy = instrument_id.split('-').next().unwrap_or("");
                    coins.push(format!("OKX:{}", instrument_id.replace("-", "")));
                }
            }
        }
    }

    if coins.is_empty() {
        println!("USDT spot piyasa çiftleri bulunamadı. API yanıtını kontrol edin.");
        println!("API Yanıtı: {}", text);
        return Ok(());
    }

    coins.sort_by(|a, b| {
        let a_name = a.split(':').nth(1).unwrap_or("").trim_end_matches("USDT");
        let b_name = b.split(':').nth(1).unwrap_or("").trim_end_matches("USDT");
        let a_numeric = a_name
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();
        let b_numeric = b_name
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();

        if !a_numeric.is_empty() && !b_numeric.is_empty() {
            let a_num: u32 = a_numeric.parse().unwrap_or(0);
            let b_num: u32 = b_numeric.parse().unwrap_or(0);
            if a_num != b_num {
                return b_num.cmp(&a_num);
            }
        }

        if a_numeric.is_empty() != b_numeric.is_empty() {
            return if a_numeric.is_empty() {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }

        a_name.cmp(b_name)
    });

    let mut file = File::create("okx_usdt_markets.txt")?;
    for coin in &coins {
        writeln!(file, "{}", coin)?;
    }

    println!("OKX USDT spot piyasa listesi başarıyla oluşturuldu ve kaydedildi.");
    println!(
        "Toplam {} adet USDT spot piyasa çifti bulundu.",
        coins.len()
    );

    Ok(())
}
