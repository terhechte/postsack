mod options {
    use std::path::PathBuf;

    use clap::AppSettings;

    #[derive(Debug, clap::Parser)]
    #[clap(name = "postsack", about = "Control the postsack mail processor", version = clap::crate_version!())]
    #[clap(setting = AppSettings::SubcommandRequiredElseHelp)]
    pub struct Args {
        /// Enable tracing for all components.
        #[clap(short = 'v', long)]
        pub verbose: bool,

        #[clap(subcommand)]
        pub cmds: SubCommands,
    }

    #[derive(Debug, clap::Parser)]
    pub enum SubCommands {
        /// Import emails and store them into a database.
        Import {
            /// The path to which to write the database containing all imported data.
            /// 
            /// Note that it will be overwritten unconditionally.
            #[clap(short = 's', long, default_value = "./postsack.sqlite")]
            database: PathBuf,

            /// Emails to be considered your own emails for filtering by Sender.
            #[clap(
                short = 'e',
                long,
                multiple_occurrences = true,
                multiple_values = false
            )]
            sender_email: Vec<String>,

            /// The kind of formats available for import, valid values are 'apple', 'gmailvault' and 'mbox'.
            #[clap(parse(try_from_str = parse_format))]
            email_format: ps_core::FormatType,

            /// The path in which all emails are stored.
            emails_folder: PathBuf,
        },
    }

    fn parse_format(s: &str) -> Result<ps_core::FormatType, String> {
        use ps_core::FormatType::*;
        Ok(match s {
            "apple" | "Apple Mail" => AppleMail,
            "gmailvault" | "Gmail Vault Download" => GmailVault,
            "mbox" | "Mbox" => Mbox,
            unknown => return Err(format!("'{}' isn't a valid format", unknown)),
        })
    }
}

use clap::Parser;
use options::{Args, SubCommands};
use ps_core::eyre;

fn main() -> eyre::Result<()> {
    let args = Args::parse();
    if args.verbose {
        ps_core::setup_tracing();
    }

    match args.cmds {
        SubCommands::Import {
            database,
            sender_email,
            email_format,
            emails_folder,
        } => {
            use ps_core::{Importerlike, DatabaseLike};
            if !emails_folder.is_dir() {
                eyre::bail!("The mails directory at '{}' isn't accessible", emails_folder.display())
            }
            let config = ps_core::Config::new(
                Some(database.clone()),
                emails_folder,
                sender_email,
                email_format,
            )?;
            let importer = ps_importer::mbox_importer(config);
            let database = ps_database::Database::new(&database)?;
            let (_messages_ignored_tb_revised_once_there_is_more_feedback, handle) = importer.import(database).unwrap();
            handle.join().expect("no panic")?;
        }
    };
    Ok(())
}
