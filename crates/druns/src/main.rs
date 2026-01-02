use clap::Parser;
use druns::app::App;
use druns::cli::{Args, Subcommands};
use druns::analytics;
use druns::batch;
use druns::gfx_demo::GfxDemoApp;
use druns::multi;
use druns::xargs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match &args.subcommand {
        Subcommands::Run { command } => {
            let command = command.clone();
            if command.is_empty() {
                eprintln!("No command provided");
                std::process::exit(1);
            }

            let mut app = App::new(args, command).await?;
            app.run().await?;
            let status = app.cleanup().await?;

            std::process::exit(status.code().unwrap_or(1));
        }
        Subcommands::GfxDemo => {
            let mut demo = GfxDemoApp::new(args).await?;
            demo.run().await?;
            demo.cleanup().await?;
        }
        Subcommands::Multi { multi } => {
            multi::handle_multi(multi).await?;
        }
        Subcommands::Xargs {
            command,
            max_procs,
            max_args,
            arg_file,
            delimiter,
            null,
            replace_str,
            no_tui,
            buffer,
            stdin_file,
            stdin_null,
        } => {
            // Read input items
            let items = xargs::read_input_items(arg_file.clone(), *delimiter, *null)?;

            if items.is_empty() {
                eprintln!("No input items provided");
                std::process::exit(1);
            }

            // Check if no-tui mode is enabled
            if *no_tui {
                // Execute without TUI
                if let Some(procs) = max_procs {
                    if *procs > 1 {
                        // Parallel execution without TUI
                        let batches = xargs::batch_items(items, *max_args);
                        xargs::handle_xargs_no_tui_parallel(
                            command.clone(),
                            replace_str.clone(),
                            batches,
                            *procs,
                            *buffer,
                            stdin_file.clone(),
                            *stdin_null,
                        )
                        .await?;
                    } else {
                        // Single execution without TUI
                        let batches = xargs::batch_items(items, *max_args);
                        for batch in batches {
                            xargs::handle_xargs_no_tui_single(
                                command.clone(),
                                replace_str.clone(),
                                batch,
                                *buffer,
                                stdin_file.clone(),
                                *stdin_null,
                            )
                            .await?;
                        }
                    }
                } else {
                    // No max_procs - sequential execution without TUI
                    let batches = xargs::batch_items(items, *max_args);
                    for batch in batches {
                        xargs::handle_xargs_no_tui_single(
                            command.clone(),
                            replace_str.clone(),
                            batch,
                            *buffer,
                            stdin_file.clone(),
                            *stdin_null,
                        )
                        .await?;
                    }
                }
            } else {
                // Execute with TUI (original behavior)
                if let Some(procs) = max_procs {
                    if *procs > 1 {
                        // Multi-job parallel execution
                        let batches = xargs::batch_items(items, *max_args);
                        xargs::handle_xargs_multi(
                            command.clone(),
                            replace_str.clone(),
                            batches,
                            *procs,
                        )
                        .await?;
                    } else {
                        // Single job execution with batching if needed
                        let batches = xargs::batch_items(items, *max_args);
                        if batches.len() == 1 {
                            xargs::handle_xargs_single(args.clone(), command.clone(), replace_str.clone(), batches[0].clone()).await?;
                        } else {
                            // Multiple batches with single job - run sequentially with TUI
                            for batch in batches {
                                xargs::handle_xargs_single(args.clone(), command.clone(), replace_str.clone(), batch).await?;
                            }
                        }
                    }
                } else {
                    // No max_procs specified - single job execution
                    let batches = xargs::batch_items(items, *max_args);
                    if batches.len() == 1 {
                        xargs::handle_xargs_single(args.clone(), command.clone(), replace_str.clone(), batches[0].clone()).await?;
                    } else {
                        // Multiple batches - run sequentially
                        for batch in batches {
                            xargs::handle_xargs_single(args.clone(), command.clone(), replace_str.clone(), batch).await?;
                        }
                    }
                }
            }
        }
        Subcommands::Analytics { analytics } => {
            analytics::handle_analytics(analytics).await?;
        }
        Subcommands::Batch { batch } => {
            batch::handle_batch(batch).await?;
        }
        Subcommands::Parallel { parallel } => {
            eprintln!("Parallel {:?} not implemented yet", parallel);
            std::process::exit(1);
        }
        Subcommands::Template { template } => {
            eprintln!("Template {:?} not implemented yet", template);
            std::process::exit(1);
        }
        Subcommands::Workflow { workflow } => {
            eprintln!("Workflow {:?} not implemented yet", workflow);
            std::process::exit(1);
        }
        Subcommands::Disk { disk } => {
            eprintln!("Disk {:?} not implemented yet", disk);
            std::process::exit(1);
        }
        Subcommands::Config { config } => {
            eprintln!("Config {:?} not implemented yet", config);
            std::process::exit(1);
        }
        Subcommands::Theme { theme } => {
            eprintln!("Theme {:?} not implemented yet", theme);
            std::process::exit(1);
        }
    }

    Ok(())
}
