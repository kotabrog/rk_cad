//! ファイル全体のexport処理

use super::StepFile;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum StepFileExportError {
    #[error("IO error: {0}")]
    Io(String),
}

impl From<std::io::Error> for StepFileExportError {
    fn from(error: std::io::Error) -> Self {
        StepFileExportError::Io(error.to_string())
    }
}

/// StepFile構造体をSTEPファイル形式の文字列に変換する
pub fn export_step_file(step_file: &StepFile) -> Result<String, StepFileExportError> {
    let mut output = String::new();

    // HEADER セクション (ISO-10303-21; と HEADER; を含む)
    for header_line in &step_file.header {
        output.push_str(header_line);
        output.push('\n');
    }
    output.push_str("ENDSEC;\n");

    // DATA セクション
    output.push_str("DATA;\n");
    for entity_line in &step_file.entities {
        output.push_str(entity_line);
        output.push('\n');
    }
    output.push_str("ENDSEC;\n");

    // TRAILER セクション (END-ISO-10303-21; など)
    for trailer_line in &step_file.trailer {
        output.push_str(trailer_line);
        output.push('\n');
    }

    Ok(output)
}

/// StepFile構造体をファイルに書き出す
pub fn export_step_file_to_path(
    step_file: &StepFile,
    path: &std::path::Path,
) -> Result<(), StepFileExportError> {
    use std::io::Write;

    let content = export_step_file(step_file)?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_step_file() {
        let step_file = StepFile {
            header: vec![
                "ISO-10303-21;".to_string(),
                "HEADER;".to_string(),
                "FILE_DESCRIPTION(('STEP AP203'), '1');".to_string(),
                "FILE_NAME('test.stp', '2023-10-01T12:00:00', ('Author'), ('Organization'), 'Description');".to_string(),
                "FILE_SCHEMA(('AP203'));".to_string(),
            ],
            entities: vec![
                "#1 = PRODUCT('Product1', 'Description1');".to_string(),
                "#2 = PRODUCT('Product2', 'Description2');".to_string(),
            ],
            trailer: vec![
                "END-ISO-10303-21;".to_string(),
            ],
        };

        let result = export_step_file(&step_file).unwrap();

        // デバッグ出力
        println!("Exported STEP file:\n{}", result);

        // 基本的な構造を確認
        assert!(result.starts_with("ISO-10303-21;\n"));
        assert!(result.contains("HEADER;\n"));
        assert!(result.contains("FILE_DESCRIPTION(('STEP AP203'), '1');"));
        assert!(result.contains("FILE_NAME('test.stp', '2023-10-01T12:00:00', ('Author'), ('Organization'), 'Description');"));
        assert!(result.contains("FILE_SCHEMA(('AP203'));"));
        assert!(result.contains("ENDSEC;\n"));
        assert!(result.contains("DATA;\n"));
        assert!(result.contains("#1 = PRODUCT('Product1', 'Description1');"));
        assert!(result.contains("#2 = PRODUCT('Product2', 'Description2');"));
        assert!(result.ends_with("END-ISO-10303-21;\n"));

        // ISO-10303-21とHEADER;が重複していないことを確認
        let iso_count = result.matches("ISO-10303-21;").count();
        let header_count = result.matches("HEADER;").count();
        assert_eq!(iso_count, 2); // 先頭と末尾
        assert_eq!(header_count, 1);
    }

    #[test]
    fn test_export_step_file_and_reimport() {
        use super::super::importer::parse_step_file;

        let original = StepFile {
            header: vec![
                "ISO-10303-21;".to_string(),
                "HEADER;".to_string(),
                "FILE_DESCRIPTION(('STEP AP203'), '1');".to_string(),
                "FILE_NAME('test.stp', '2023-10-01T12:00:00', ('Author'), ('Organization'), 'Description');".to_string(),
                "FILE_SCHEMA(('AP203'));".to_string(),
            ],
            entities: vec![
                "#1 = PRODUCT('Product1', 'Description1');".to_string(),
                "#2 = PRODUCT('Product2', 'Description2');".to_string(),
                "#3 = CARTESIAN_POINT('', (0.0, 0.0, 0.0));".to_string(),
            ],
            trailer: vec![
                "END-ISO-10303-21;".to_string(),
            ],
        };

        // エクスポート
        let exported = export_step_file(&original).unwrap();
        println!("Exported:\n{}", exported);

        // 再インポート
        let reimported = parse_step_file(&exported).unwrap();
        println!("Reimported header: {:?}", reimported.header);
        println!("Original header: {:?}", original.header);

        // 内容が一致することを確認
        assert_eq!(original.header, reimported.header);
        assert_eq!(original.entities, reimported.entities);
        assert_eq!(original.trailer, reimported.trailer);
    }

    #[test]
    fn test_export_step_file_import_then_export() {
        use super::super::importer::parse_step_file;

        // importer.rsのparse_step_file_normalテストと同じSTEPファイルデータを使用
        // ただし、空白行やインデントを正規化
        let src = "ISO-10303-21;\n\
HEADER;\n\
FILE_DESCRIPTION(('STEP AP203'), '1');\n\
FILE_NAME('test.stp', '2023-10-01T12:00:00', ('Author'), ('Organization'), 'Description');\n\
FILE_SCHEMA(('AP203'));\n\
ENDSEC;\n\
DATA;\n\
#1 = PRODUCT('Product1', 'Description1');\n\
#2 = PRODUCT('Product2', Description2');\n\
ENDSEC;\n\
END-ISO-10303-21;\n";

        // インポート
        let imported = parse_step_file(src).unwrap();
        println!("Imported header: {:?}", imported.header);
        println!("Imported entities: {:?}", imported.entities);
        println!("Imported trailer: {:?}", imported.trailer);

        // エクスポート
        let exported = export_step_file(&imported).unwrap();
        println!("Exported:\n{}", exported);

        // エクスポートされた内容が元のSTEPファイルと完全に一致することを確認
        assert_eq!(src, exported);

        // 再度インポートして内容が保持されていることも確認
        let reimported = parse_step_file(&exported).unwrap();
        assert_eq!(imported.header, reimported.header);
        assert_eq!(imported.entities, reimported.entities);
        assert_eq!(imported.trailer, reimported.trailer);
    }
}
