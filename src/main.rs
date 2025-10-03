use clap::Parser;
use druns::app::App;
use druns::cli::Args;
use druns::gfx_demo::GfxDemoApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.command.is_empty() {
        eprintln!("No command provided");
        std::process::exit(1);
    }

    let (is_run, command) = if args.command[0] == "run" {
        (true, args.command[1..].to_vec())
    } else if args.command[0] == "gfx-demo" {
        (false, vec![])
    } else {
        (true, args.command.clone())
    };

    if is_run {
        if command.is_empty() {
            eprintln!("No command provided");
            std::process::exit(1);
        }

        let mut app = App::new(args, command).await?;
        app.run().await?;
        let status = app.cleanup().await?;

        std::process::exit(status.code().unwrap_or(1));
    } else {
        let mut demo = GfxDemoApp::new(args).await?;
        demo.run().await?;
        demo.cleanup().await?;
    }

    Ok(())
}

