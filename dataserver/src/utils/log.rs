use tokio::signal;

pub fn logo() {
    println!(
        r"
      __________ .__                          __________ .__ .__            __         
      \______   \|  |  _____     ____    ____ \______   \|__||  |    ____ _/  |_       
       |     ___/|  |  \__  \   /    \ _/ __ \ |     ___/|  ||  |   /  _ \\   __\      
       |    |    |  |__ / __ \_|   |  \\  ___/ |    |    |  ||  |__(  <_> )|  |        
       |____|    |____/(____  /|___|  / \___  >|____|    |__||____/ \____/ |__|        
                            \/      \/      \/                                         
      ________            __             _________                                     
      \______ \  _____  _/  |_ _____    /   _____/  ____ _______ ___  __  ____ _______ 
       |    |  \ \__  \ \   __\\__  \   \_____  \ _/ __ \\_  __ \\  \/ /_/ __ \\_  __ \
       |    `   \ / __ \_|  |   / __ \_ /        \\  ___/ |  | \/ \   / \  ___/ |  | \/
      /_______  /(____  /|__|  (____  //_______  / \___  >|__|     \_/   \___  >|__|   
              \/      \/            \/         \/      \/                    \/                                                                                                                                                                                                        
    "
    );
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
