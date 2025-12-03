use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::fs::File;
use std::io::BufReader;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearRecord {
    pub gregorian_year: i32,
    pub ganzhi: String,
    pub nian_hexagram: String,
    pub dynasty: String,
    pub person: String,
    pub yuan_raw: String,
    pub hui_raw: String,
    pub yun_raw: String,
    pub shi_raw: String,
    pub xun_raw: String,
}

// Global data storage
pub static YEAR_DATA: Lazy<RwLock<HashMap<i32, YearRecord>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

pub fn load_data(path: &str) -> anyhow::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let records: Vec<YearRecord> = serde_json::from_reader(reader)?;
    
    let mut map = YEAR_DATA.write().unwrap();
    for record in records {
        map.insert(record.gregorian_year, record);
    }
    println!("Loaded {} year records.", map.len());
    Ok(())
}

pub fn get_year_record(year: i32) -> Option<YearRecord> {
    let map = YEAR_DATA.read().unwrap();
    map.get(&year).cloned()
}
