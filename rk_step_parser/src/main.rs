use std::env;
use std::fs;
use std::process;

use rk_step_parser::{parse_step, write_step};

fn main() {
    // コマンドライン引数の取得。引数が不十分な場合は使い方を表示して終了
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input_file.step> <output_file.step>", args[0]);
        process::exit(1);
    }
    let input_file = &args[1];
    let output_file = &args[2];

    // 入力ファイルの内容を文字列として読み込む
    let content = match fs::read_to_string(input_file) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Error reading file {}: {}", input_file, err);
            process::exit(1);
        }
    };

    // STEPパーサでCadModelに変換する
    let model = match parse_step(&content) {
        Ok(model) => model,
        Err(err) => {
            eprintln!("Error parsing STEP file: {}", err);
            process::exit(1);
        }
    };

    // 変換したCadModelからSTEP形式の文字列を生成
    let output_content = write_step(&model);

    // 出力ファイルに書き出し
    if let Err(err) = fs::write(output_file, output_content) {
        eprintln!("Error writing file {}: {}", output_file, err);
        process::exit(1);
    }

    println!(
        "STEP file processed successfully. Output written to {}",
        output_file
    );
}
