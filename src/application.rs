use eframe::Frame;
use egui::Context;
use native_dialog::FileDialog;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Default)]
pub struct DragonShieldApplication {
    input_path: PathBuf,
    output_path: String,
    message: String,
}

impl DragonShieldApplication {
    pub fn new() -> Self {
        Default::default()
    }
}

impl eframe::App for DragonShieldApplication {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Input CSV...").clicked() {
                    let path = pick_a_file();
                    match path {
                        None => {}
                        Some(path) => {
                            self.input_path = path;
                        }
                    }
                };
                ui.label(self.input_path.to_str().unwrap());
            });

            ui.horizontal(|ui| {
                ui.label("Output path: ");
                ui.text_edit_singleline(&mut self.output_path)
            });

            ui.horizontal(|ui| {
                ui.label("Messages: ");
                ui.label(&self.message);
            });

            if ui.button("Convert").clicked() {
                if !self.input_path.exists() {
                    self.message = String::from("No input path!");
                    return;
                }

                if self.output_path.is_empty() {
                    self.message = String::from("No output input path!");
                    return;
                }
                convert(&self.input_path, self.output_path.as_str()).expect("Failed to convert!");
            }
        });
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Record {
    #[serde(rename(deserialize = "Quantity", serialize = "Count"))]
    quantity: String,
    #[serde(rename(deserialize = "Card Name", serialize = "Name"))]
    card_name: String,
    #[serde(rename(deserialize = "Set Code", serialize = "Edition"))]
    set_code: String,
    #[serde(rename(deserialize = "Card Number", serialize = "Collector Number"))]
    card_number: String,
}

fn convert(input_path: &PathBuf, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut buf_reader = BufReader::new(File::open(input_path)?);
    buf_reader.read_line(&mut String::new())?; // skip weird first line that DragonShield exports
    let mut reader = csv::Reader::from_reader(buf_reader);
    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(output_path)
        .unwrap();
    let mut writer = csv::Writer::from_writer(output_file);
    for result in reader.deserialize() {
        let record: Record = result?;
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}

fn pick_a_file() -> Option<PathBuf> {
    let path = FileDialog::new()
        .set_location("~/Desktop")
        .add_filter("CSV", &["csv"])
        .show_open_single_file()
        .unwrap();

    path
}

#[cfg(test)]
mod tests {
    use crate::application::convert;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn should_convert_without_error() {
        let mut input_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        input_file.push("resources/tests/DragonShieldInput.csv");
        let mut output_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        output_file.push("resources/tests/output.csv");
        if output_file.exists() {
            fs::remove_file(&output_file).unwrap();
        }
        convert(&input_file, output_file.to_str().unwrap()).expect("Something went wrong");
    }
}