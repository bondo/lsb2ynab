use std::path::PathBuf;

use clap::Parser;
use rfd::FileDialog;
use serde::{ser::SerializeStruct, Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct MonetaryValue(String);

impl<'de> Deserialize<'de> for MonetaryValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(MonetaryValue(raw.replace(".", "").replace(",", ".")))
    }
}

#[derive(Debug, Serialize)]
struct DateValue(String);

impl<'de> Deserialize<'de> for DateValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let mut parts: Vec<&str> = raw.split("-").collect();
        parts.reverse();
        Ok(DateValue(parts.join("-")))
    }
}

#[derive(Debug, Deserialize)]
struct Row {
    date: DateValue,
    entry: String,
    value: MonetaryValue,
    _running_total: String,
    _currency: String,
}

impl Serialize for Row {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Row", 4)?;
        s.serialize_field("Date", &self.date)?;
        s.serialize_field("Payee", &self.entry)?;
        s.serialize_field("Memo", "")?;
        s.serialize_field("Amount", &self.value)?;
        s.end()
    }
}

#[derive(Parser, Debug)]
struct Args {
    input: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_path(args.input)
        .expect("Failed to open file for reading");

    let Some(output_path) = FileDialog::new()
        .set_file_name(format!(
            "ynab-{}.csv",
            chrono::Local::now().format("%Y-%m-%d-%H-%M-%S")
        ))
        .save_file()
    else {
        return;
    };

    let mut writer = csv::Writer::from_path(output_path).expect("Failed to open file for writing");

    for record in reader.deserialize::<Row>() {
        match record {
            Ok(record) => {
                writer.serialize(record).expect("Should be writable");
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }
}
