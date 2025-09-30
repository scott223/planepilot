use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use tracing::event;

pub mod xplane;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();    
    utils::start_tracing_subscriber();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    logo();

    tokio::select! {
        //_ = pp_planeconnector::run_app(&service_adresses) => { },
        //_ = pp_dataserver::run_app(&service_adresses) => { },
        _ = xplane::listen_to_xplane() => {},

        // process that runs a terminal, that looks for input (eg "q" press)
        // this is the process that will run to completion and then the tokio::select will cancel the rest
        _ = run_terminal() => { }
    }

    event!(tracing::Level::INFO, "Planepilot closed");
}

// listents to terminal inputs, and breaks on "q"
async fn run_terminal() -> Result<(), ()> {
    let mut reader = EventStream::new();

    loop {
        tokio::select! {
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(event)) => {

                        if event == Event::Key(KeyCode::Char('q').into()) {
                            break;
                        }
                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => break,
                }
            }
        };
    }

    Ok(())
}

fn logo() {
    println!(
        r"
                                |
            ____________________|____________________
                           \  |   |  /
                            '.#####.'
                             /'#_#'\
                           O'   O   'O 
__________.__                      __________.__.__          __   
\______   |  | _____    ____   ____\______   |__|  |   _____/  |_ 
 |     ___|  | \__  \  /    \_/ __ \|     ___|  |  |  /  _ \   __\
 |    |   |  |__/ __ \|   |  \  ___/|    |   |  |  |_(  <_> |  |  
 |____|   |____(____  |___|  /\___  |____|   |__|____/\____/|__|  
                    \/     \/     \/                      v0.2                            
    "
    );
}