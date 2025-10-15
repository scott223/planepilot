use tracing::event;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }

    planepilot::utils::start_tracing_subscriber();
    logo();

    planepilot::run_app().await.unwrap();

    event!(tracing::Level::INFO, "Planepilot closed");
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
