//! 簡易 CLI:  STEP → Model (parse) / Model → STEP (write)
//
// 依存クレート：
//   clap   = { version = "4", features = ["derive"] }
//   anyhow = "1"
use std::{fs::File, path::PathBuf};

use clap::{Parser, Subcommand};

use rk_step_parser::{
    build_graph, export_model, import_cube, parse_step_file, resolve_refs, write_step_file,
};

/// rkstep CLI
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// STEP → Model へ変換して要素数を表示
    Parse {
        /// 入力 STEP ファイル
        input: PathBuf,
    },
    /// 入力 STEP → Model → 新 STEP として書き出し
    Write {
        /// 入力 STEP
        input: PathBuf,
        /// 出力 STEP
        output: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        /* -------------------- parse -------------------- */
        Cmd::Parse { input } => {
            let src = std::fs::read_to_string(&input)?;
            let sf = parse_step_file(&src)?;
            let g = build_graph(&sf.entities);
            resolve_refs(&g);
            let model = import_cube(&g)?;

            println!("vertices: {}", model.vertices().count());
            println!("edges   : {}", model.edges().count());
            println!("faces   : {}", model.faces().count());
            println!("solids  : {}", model.solids().count());
        }

        /* -------------------- write -------------------- */
        Cmd::Write { input, output } => {
            let src = std::fs::read_to_string(&input)?;
            let sf = parse_step_file(&src)?;
            let g = build_graph(&sf.entities);
            resolve_refs(&g);
            let model = import_cube(&g)?;

            let out_sf = export_model(&model);
            let mut f = File::create(&output)?;
            write_step_file(&out_sf, &mut f)?;
            println!("wrote {}", output.display());
        }
    }

    Ok(())
}
