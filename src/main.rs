use axum::{response::Html, routing::post, Router};
use tokio::process::Command;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new().route("/toggle", post(toggle_media));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}

async fn toggle_media() -> Html<String> {
    println!("Toggling the play button");
    match Command::new("xdotool")
        .arg("key")
        .arg("XF86AudioPlay")
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                info!("xdotool succeeded");
                Html("<h1>Toggled media successfully</h1>".into())
            } else {
                // Convert stderr bytes to a String (lossy so we don’t panic on bad UTF-8)
                let err_text = String::from_utf8_lossy(&output.stderr);
                error!("xdotool failed: {}", err_text);
                Html(format!(
                    "<h1>Failed to toggle media</h1>\
                     <h2>xdotool exited with {}</h2>\
                     <pre>{}</pre>",
                    output.status, err_text
                ))
            }
        }
        Err(e) => {
            // If the Command itself couldn't launch (e.g., xdotool not found)
            error!("Failed to spawn xdotool: {}", e);
            Html(format!(
                "<h1>Error launching xdotool</h1>\
                 <pre>{}</pre>",
                e
            ))
        }
    }
}
