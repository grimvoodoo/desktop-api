use axum::{response::Html, routing::post, Router};
use tokio::process::Command;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    start_ydotoold().await;
    tracing_subscriber::fmt::init();
    let app = Router::new().route("/toggle", post(toggle_media));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}

// Start the ydotool daemon
async fn start_ydotoold() {
    match Command::new("ydotoold")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => println!("Started ydotoold"),
        Err(e) => eprintln!("Failed to start ydotoold: {}", e),
    }
}

async fn toggle_media() -> Html<String> {
    println!("Toggling the play button");
    match Command::new("ydotool")
        .arg("key")
        .arg("164")
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                info!("ydotool succeeded");
                Html("<h1>Toggled media successfully</h1>".into())
            } else {
                // Convert stderr bytes to a String (lossy so we don’t panic on bad UTF-8)
                let err_text = String::from_utf8_lossy(&output.stderr);
                error!("ydotool failed: {}", err_text);
                Html(format!(
                    "<h1>Failed to toggle media</h1>\
                     <h2>ydotool exited with {}</h2>\
                     <pre>{}</pre>",
                    output.status, err_text
                ))
            }
        }
        Err(e) => {
            // If the Command itself couldn't launch (e.g., xdotool not found)
            error!("Failed to spawn ydotool: {}", e);
            Html(format!(
                "<h1>Error launching ydotool</h1>\
                 <pre>{}</pre>",
                e
            ))
        }
    }
}
