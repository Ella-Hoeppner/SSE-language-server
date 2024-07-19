use crate::server::Backend;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tower_lsp::{LspService, Server};

async fn start_server_terminal() {
  let stdin = tokio::io::stdin();
  let stdout = tokio::io::stdout();

  let (service, socket) = LspService::new(|client| Backend { client });

  Server::new(stdin, stdout, socket).serve(service).await;
}

fn annotate_content_length<'a>(message: &'a str) -> String {
  let content_length = message.len();
  format!("Content-Length: {}\r\n\r\n{}", content_length, message)
}

async fn start_server_with_messages(
  messages: Vec<&'static str>,
) -> Vec<String> {
  let (mut stdin_writer, stdin_reader) = tokio::io::duplex(1024);
  let (stdout_writer, stdout_reader) = tokio::io::duplex(1024);
  let output = Arc::new(Mutex::new(Vec::new()));

  // Spawn a task to handle server input and output
  let output_clone = Arc::clone(&output);
  tokio::spawn(async move {
    let mut reader = BufReader::new(stdout_reader);
    let mut buffer = String::new();

    for message in messages {
      // Write the message to stdin
      stdin_writer
        .write_all(annotate_content_length(message).as_bytes())
        .await
        .unwrap();

      // Read the response
      buffer.clear();
      reader.read_line(&mut buffer).await.unwrap(); // Read Content-Length header
      let content_length: usize =
        buffer["Content-Length: ".len()..].trim().parse().unwrap();

      reader.read_line(&mut buffer).await.unwrap(); // Read empty line

      let mut response = vec![0; content_length];
      reader.read_exact(&mut response).await.unwrap(); // Read the response body

      let response_str = String::from_utf8(response).unwrap();
      let mut output_lock = output_clone.lock().unwrap();
      output_lock.push(response_str);
    }
  });

  let (service, socket) = LspService::new(|client| Backend { client });

  Server::new(stdin_reader, stdout_writer, socket)
    .serve(service)
    .await;

  // Return the collected output
  let output = Arc::try_unwrap(output)
    .expect("Failed to unwrap Arc")
    .into_inner()
    .unwrap();
  output
}

const INIT_MESSAGE: &'static str = r#"{
    "jsonrpc":"2.0",
    "method":"initialize",
    "params":{
        "processId": null,
        "rootUri": null,
        "capabilities": {}
    },
    "id":1
  }"#;

#[tokio::test]
async fn initialize() {
  let out = start_server_with_messages(vec![INIT_MESSAGE]).await;
  assert_eq!(
    out,
    vec![
      r#"{"jsonrpc":"2.0","result":{"capabilities":{"hoverProvider":true}},"id":1}"#
    ]
  )
}

#[tokio::test]
async fn initialize_and_hover() {
  let out = start_server_with_messages(vec![
    INIT_MESSAGE,
    r#"{
        "jsonrpc": "2.0",
        "method": "textDocument/hover",
        "params": {
          "textDocument": {
            "uri": "file:///path/to/your/file"
          },
          "position": {
            "line": 0,
            "character": 1
          }
        },
        "id": 2
      }"#,
  ])
  .await;
  assert_eq!(
    out,
    vec![
      r#"{"jsonrpc":"2.0","result":{"capabilities":{"hoverProvider":true}},"id":1}"#,
      r#"{"jsonrpc":"2.0","result":{"contents":"hovering!!"},"id":2}"#
    ]
  )
}
