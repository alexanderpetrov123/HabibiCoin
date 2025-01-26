use axum::{
    extract::{State},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use hyper::Server;
use serde_json::json;
use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::services::ServeDir;

// Fetch ETH price from CoinGecko API
async fn fetch_eth_price() -> Result<f64, Box<dyn std::error::Error>> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";
    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let eth_price = response["ethereum"]["usd"]
        .as_f64()
        .ok_or("Failed to fetch ETH price")?;
    Ok(eth_price)
}

// Fetch Silver price from Metals API
async fn fetch_silver_price() -> Result<f64, Box<dyn std::error::Error>> {
    let url = "https://metals-api.com/api/latest?access_key=YOUR_ACCESS_KEY&symbols=XAG";
    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let silver_price = response["rates"]["XAG"]
        .as_f64()
        .ok_or("Failed to fetch silver price")?;
    Ok(silver_price)
}

// Increment HabibiCoins
async fn increment_counter(State(state): State<Arc<Mutex<i32>>>) -> Json<serde_json::Value> {
    let mut counter = state.lock().unwrap();
    *counter += 1;

    let eth_price = 2000.0; // Example ETH price
    let silver_price = 1000.0; // Example Silver price

    let eth_to_usd_value = (*counter as f64) * eth_price;
    let silver_weight = eth_to_usd_value / silver_price;
    let camels_needed = (silver_weight / 10.0).ceil() as u32;

    let message = format!(
        "Yo Habibi, your {} HabibiCoins are enough for {} camels carrying {:.1} kg of silver!",
        counter, camels_needed, silver_weight
    );

    Json(json!({ "counter": *counter, "message": message }))
}

// Fetch current conversion state
async fn convert(State(state): State<Arc<Mutex<i32>>>) -> Json<serde_json::Value> {
    let counter = *state.lock().unwrap();

    let eth_price = fetch_eth_price().await.unwrap_or(2000.0);
    let silver_price = fetch_silver_price().await.unwrap_or(1000.0);

    let eth_to_usd_value = (counter as f64) * eth_price;
    let silver_weight = eth_to_usd_value / silver_price;
    let camels_needed = (silver_weight / 10.0).ceil() as u32;

    let message = format!(
        "Yo Habibi, your {} HabibiCoins are enough for {} camels carrying {:.1} kg of silver!",
        counter, camels_needed, silver_weight
    );

    Json(json!({ "counter": counter, "message": message }))
}

// Serve the HTML page
async fn serve_html() -> Html<String> {
    let eth_price = fetch_eth_price().await.unwrap_or(2000.0); 
    let html_template = include_str!("index.html"); 

    let html_content = html_template.replace("{{eth_price}}", &format!("{:.2}", eth_price));

    Html(html_content)
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(0));

    let app = Router::new()
        .route("/", get(serve_html)) // Serve the main HTML page
        .route("/increment", post(increment_counter)) // Increment HabibiCoin counter
        .route("/convert", get(convert)) // Fetch the current counter and conversion state
        .nest_service("/static", axum::routing::get_service(ServeDir::new("static"))) // Serve static files
        .with_state(shared_state);

    // Get the port from the environment variable, default to 3000
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("Failed to parse PORT");

    let addr = SocketAddr::from(([0, 0, 0, 0], port)); // Bind to 0.0.0.0 for external access
    println!("Server running at http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
