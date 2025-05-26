use axum::Router;


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/hello", axum::routing::get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind TCP listener");

    let server = axum::serve(listener, app);
        

    println!("Server running on http://localhost:3000");

    server.await.expect("Failed to start server");

}
