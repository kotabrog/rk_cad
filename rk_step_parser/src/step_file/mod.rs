mod exporter;
mod importer;

pub use exporter::{export_step_file, export_step_file_to_path, StepFileExportError};
pub use importer::{parse_step_file, StepFileParseError};

/// STEP ファイルを 3 つのセクションに分割して保持する
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepFile {
    pub header: Vec<String>,   // ISO-10303-21 HEADER;
    pub entities: Vec<String>, // DATA; 〜 ENDSEC; までの各エンティティ行
    pub trailer: Vec<String>,  // END-ISO-10303-21 以降
}
