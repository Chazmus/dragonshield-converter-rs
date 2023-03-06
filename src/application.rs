use eframe::Frame;
use egui::Context;
use pollster::FutureExt as _;
use rfd::AsyncFileDialog;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::BufRead;

#[derive(Default)]
pub struct DragonShieldApplication {
    input_path: String,
    input_data: Vec<u8>,
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
                    let (path, data) = pick_a_file().block_on();
                    self.input_path = path;
                    self.input_data = data;
                };
                ui.label(self.input_path.as_str());
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
                if self.input_data.is_empty() {
                    self.message = String::from("No input csv!");
                    return;
                }

                if self.output_path.is_empty() {
                    self.message = String::from("No output input path!");
                    return;
                }
                convert(&self.input_data, self.output_path.as_str()).expect("Failed to convert!");
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

fn convert(input_data: &Vec<u8>, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut c = input_data.as_slice();
    c.read_line(&mut String::new())?; // skip weird first line that DragonShield exports
    let mut reader = csv::Reader::from_reader(c);
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

async fn pick_a_file() -> (String, Vec<u8>) {
    let file = AsyncFileDialog::new()
        .add_filter("csv", &["csv"])
        .set_directory("~")
        .pick_file()
        .await
        .unwrap();
    let file_name = file.file_name();
    let data = file.read().await;
    (file_name, data)
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
        // get data from file as a vector of u8
        let data = fs::read(input_file).unwrap();
        let mut output_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        output_file.push("resources/tests/output.csv");
        convert(&data, output_file.to_str().unwrap()).expect("Something went wrong");
        if output_file.exists() {
            fs::remove_file(&output_file).unwrap();
        }
    }
}
