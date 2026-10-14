use clap::{CommandFactory, FromArgMatches};
use mediaagent::cli::{Cli, dispatch};

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    let wants_json = args.iter().any(|a| a == "--json");
    let matches = match Cli::command().try_get_matches_from(args) {
        Ok(c) => c,
        Err(e) => {
            if e.use_stderr() && wants_json {
                println!(
                    "{}",
                    serde_json::json!({"schema_version":1,"command":"parse","ok":false,"data":null,"errors":[mediaagent::error::Error::input(e.to_string())],"warnings":[]})
                );
                std::process::exit(2);
            }
            let exit = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            std::process::exit(exit);
        }
    };
    let cli = Cli::from_arg_matches(&matches).expect("Validated command arguments");
    let mut names = Vec::new();
    let mut current = &matches;
    while let Some((name, sub)) = current.subcommand() {
        names.push(name);
        current = sub;
    }
    let command = names.join(".");
    let exit = match dispatch(&cli).await {
        Ok((data, exit)) => {
            let (data, warnings) = mediaagent::cli::result_view(data);
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({"schema_version":1,"command":command,"ok":exit==0,"data":data,"errors":if exit==0 {vec![]} else {vec![mediaagent::error::Error::new("operation_incomplete","Inspect returned diagnostics, jobs, or review findings",exit)]},"warnings":warnings})
                );
            } else {
                println!("{}", mediaagent::cli::human_result(&data));
            }
            exit
        }
        Err(e) => {
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({"schema_version":1,"command":command,"ok":false,"data":null,"errors":[e],"warnings":[]})
                );
            } else {
                eprintln!("{e}");
            }
            e.exit
        }
    };
    std::process::exit(i32::from(exit));
}
