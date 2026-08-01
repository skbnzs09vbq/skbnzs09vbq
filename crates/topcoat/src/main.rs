//! `topcoat` — 自作 SSG (Static Site Generator) の CLI エントリーポイント。
//!
//! 現時点ではサブコマンドの受け口のみを実装し、実際のサイト生成ロジックは後続 issue で追加する。

use std::process::ExitCode;

/// `topcoat` が受け付けるサブコマンド。
enum Command {
    /// サイトをビルドする。
    Build,
}

fn parse_command(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    // args[0] はバイナリ名のため読み飛ばす。
    args.next();

    match args.next().as_deref() {
        Some("build") => Ok(Command::Build),
        Some(other) => Err(format!("unknown subcommand: {other}")),
        None => Err("missing subcommand (expected: build)".to_string()),
    }
}

fn run(command: Command) {
    match command {
        // 実際のサイト生成ロジックは後続 issue で実装する。
        Command::Build => println!("topcoat build"),
    }
}

fn main() -> ExitCode {
    match parse_command(std::env::args()) {
        Ok(command) => {
            run(command);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            eprintln!("usage: topcoat build");
            ExitCode::FAILURE
        }
    }
}
