//! `topcoat-db` の初期シードデータ投入用バイナリ（`cargo run --bin seed`）。
//!
//! `crates/topcoat/src/main.rs` の `parse_command`/`run`/`ExitCode` 構成を踏襲する。
//!
//! サブコマンド:
//! - 無引数: フルシード（[`reset::reset`] → [`insert::insert_all`]）。既存データを
//!   FK 依存順で全削除してから再投入するため、複数回実行しても結果が変わらない（冪等）。
//! - `reset`: 削除のみ（[`reset::reset`]）。

mod data;
mod insert;
mod reset;

use std::process::ExitCode;

/// `seed` が受け付けるサブコマンド。
enum Command {
    /// 既存データを削除してから再投入する（フルシード）。
    Seed,
    /// 既存データの削除のみ行う。
    Reset,
}

fn parse_command(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    // args[0] はバイナリ名のため読み飛ばす。
    args.next();

    match args.next().as_deref() {
        None => Ok(Command::Seed),
        Some("reset") => Ok(Command::Reset),
        Some(other) => Err(format!("unknown subcommand: {other}")),
    }
}

fn run(command: Command) -> ExitCode {
    let mut connection = match topcoat_db::establish_connection() {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("DB への接続に失敗しました: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = topcoat_db::run_migrations(&mut connection) {
        eprintln!("マイグレーションの適用に失敗しました: {err}");
        return ExitCode::FAILURE;
    }

    let (result, success_message) = match command {
        Command::Seed => (
            reset::reset(&mut connection).and_then(|()| insert::insert_all(&mut connection)),
            "シードデータの投入が完了しました（reset → insert）",
        ),
        Command::Reset => (
            reset::reset(&mut connection),
            "既存データの削除が完了しました",
        ),
    };

    match result {
        Ok(()) => {
            println!("{success_message}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("処理に失敗しました: {err}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    match parse_command(std::env::args()) {
        Ok(command) => run(command),
        Err(message) => {
            eprintln!("error: {message}");
            eprintln!("usage: seed [reset]");
            ExitCode::FAILURE
        }
    }
}
