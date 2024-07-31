#![windows_subsystem = "windows"]

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
    amount: MonetaryValue,
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
        s.serialize_field("Amount", &self.amount)?;
        s.end()
    }
}

fn convert(reader: impl std::io::Read, writer: impl std::io::Write) {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_reader(reader);

    let mut writer = csv::Writer::from_writer(writer);

    for record in reader.deserialize::<Row>() {
        writer
            .serialize(record.expect("Failed to parse row"))
            .expect("Failed to write row");
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file. If not provided, a file dialog will be shown.
    input: Option<PathBuf>,

    /// Output file. If not provided, a file name will be generated and placed in the folder of the input file.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let Some(input_path) = args.input.or_else(|| {
        FileDialog::new()
            .set_title("Choose CSV export from LSB")
            .add_filter("csv", &["csv"])
            .pick_file()
    }) else {
        return;
    };

    let reader = std::fs::File::open(&input_path).expect("Failed to open file for reading");

    let output_path = args.output.unwrap_or_else(|| {
        input_path.with_file_name(format!(
            "ynab {}.csv",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ))
    });

    let writer = std::fs::File::create(output_path).expect("Failed to open file for writing");

    convert(reader, writer);
}

mod tests {
    #[test]
    fn test_process_file() {
        let input = r#"20-10-2021;Test;1.234,56;456,78;EUR
15-01-2022;Test2;7,89;12,34;DKK
"#;
        let mut output = Vec::new();
        super::convert(input.as_bytes(), &mut output);
        assert_eq!(
            String::from_utf8(output).unwrap(),
            r#"Date,Payee,Memo,Amount
2021-10-20,Test,,1234.56
2022-01-15,Test2,,7.89
"#
        );
    }
}
