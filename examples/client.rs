use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

// Simple example client for interacting with the doc server via stdin/stdout
async fn stdio_client() -> Result<()> {
    // Start the stdio-server in a separate process
    let mut child = tokio::process::Command::new("cargo")
        .args(["run", "--bin", "cratedocs-mcp", "stdio"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut stdin = io::BufWriter::new(stdin);
    let mut stdout = BufReader::new(stdout);

    // Send initialize request first
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {
                "tools": {},
                "sampling": {}
            },
            "clientInfo": {
                "name": "ExampleClient",
                "version": "1.0.0"
            }
        },
        "id": 0
    });
    
    println!("Sending initialize request...");
    stdin.write_all(initialize_request.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;
    
    // Read initialize response
    let mut init_response = String::new();
    stdout.read_line(&mut init_response).await?;
    println!("Initialize response: {:?}", init_response);
    
    // Send initialized notification
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    
    println!("Sending initialized notification...");
    stdin.write_all(initialized_notification.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;
    
    // Get list of available tools first
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });
    
    println!("Sending request to list available tools...");
    stdin.write_all(list_tools_request.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;
    
    // Read tools list response
    let mut tools_response = String::new();
    stdout.read_line(&mut tools_response).await?;
    println!("Tools list response: {:?}", tools_response);
    
    // Send a request to lookup tokio crate using tools/call method
    let request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "lookup_crate",
            "arguments": {
                "crate_name": "tokio"
            }
        },
        "id": 2
    });

    println!("Sending request to look up tokio crate...");
    stdin.write_all(request.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read the response
    let mut response = String::new();
    stdout.read_line(&mut response).await?;

    println!("response {:?}", response);
    let parsed: Value = serde_json::from_str(&response)?;
    println!(
        "Received response: {}",
        serde_json::to_string_pretty(&parsed)?
    );

    // Terminate the child process
    child.kill().await?;

    Ok(())
}

// Simple example client for interacting with the doc server via HTTP/SSE
async fn http_sse_client() -> Result<()> {
    println!("Connecting to HTTP/SSE server...");

    // Create HTTP client
    let client = Client::new();

    // Create a separate task to run the server
    let _server = tokio::spawn(async {
        tokio::process::Command::new("cargo")
            .args(["run", "--bin", "cratedocs-mcp", "http"])
            .output()
            .await
            .expect("Failed to start server");
    });

    // Give the server some time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let sse_url = "http://127.0.0.1:8080/sse";

    println!("Getting session ID from SSE endpoint...");

    // For a real implementation, you would use an SSE client library
    // This is a simplified example that just gets the session ID from the first message
    let response = client.get(sse_url).send().await?;
    if !response.status().is_success() {
        println!("Error connecting to SSE endpoint: {}", response.status());
        return Ok(());
    }

    // Parse the first message to get the session ID
    // In a real implementation, you would properly handle the SSE stream
    if response.headers().get("x-accel-buffering").is_none() {
        println!("Could not get session ID from SSE endpoint");
        return Ok(());
    }
    // This is just a placeholder - in a real SSE client you would parse the actual event
    let session_id = "example_session_id".to_string();

    // Send a request to search for crates
    let request_url = format!("{}?sessionId={}", sse_url, session_id);
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "call_tool",
        "params": {
            "name": "search_crates",
            "arguments": {
                "query": "async runtime",
                "limit": 5
            }
        },
        "id": 1
    });

    println!("Sending request to search for crates...");
    let response = client.post(&request_url).json(&request_body).send().await?;

    if response.status().is_success() {
        println!("Request sent successfully");
    } else {
        println!("Error sending request: {}", response.status());
    }

    // In a real implementation, you would read the responses from the SSE stream
    println!("In a real implementation, responses would be read from the SSE stream");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Rust Documentation Server Client Example");
    println!("---------------------------------------");

    println!("\n1. Testing STDIN/STDOUT client:");
    if let Err(e) = stdio_client().await {
        println!("Error in STDIN/STDOUT client: {}", e);
    }

    //println!("\n2. Testing HTTP/SSE client:");
    //if let Err(e) = http_sse_client().await {
    //    println!("Error in HTTP/SSE client: {}", e);
    //}

    Ok(())
}
