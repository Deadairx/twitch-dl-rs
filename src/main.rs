mod cli;

fn main() {
    let cli = cli::parse_args();
    match cli.command.as_str() {
        "download" => {
            // TODO: Implement download logic
            println!("Download command invoked");
        }
        _ => {
            eprintln!("Unknown command");
        }
    }
}
