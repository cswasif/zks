//! Example resources for ZKS MCP server
//! 
//! Provides access to ZKS Protocol example code including basic connections,
//! anonymous routing, file transfers, and key generation examples.

use rmcp::model::{ResourceTemplate, ResourceContents, RawResourceTemplate};
use rmcp::ErrorData;

#[derive(Clone)]
pub struct ExamplesResource {
    zks_protocol_root: String,
}

impl ExamplesResource {
    pub fn new(zks_protocol_root: String) -> Self {
        Self { zks_protocol_root }
    }

    pub fn resources(&self) -> Vec<ResourceTemplate> {
        vec![
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://examples/basic_connection".into(),
                    name: "Basic Connection Example".into(),
                    title: None,
                    description: Some("Basic ZKS connection example".into()),
                    mime_type: Some("text/x-rust".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://examples/anonymous_connection".into(),
                    name: "Anonymous Connection Example".into(),
                    title: None,
                    description: Some("Anonymous routing example".into()),
                    mime_type: Some("text/x-rust".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://examples/file_transfer".into(),
                    name: "File Transfer Example".into(),
                    title: None,
                    description: Some("Secure file transfer example".into()),
                    mime_type: Some("text/x-rust".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://examples/keypair_generation".into(),
                    name: "Key Generation Example".into(),
                    title: None,
                    description: Some("Post-quantum key generation example".into()),
                    mime_type: Some("text/x-rust".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://examples/handshake".into(),
                    name: "Handshake Example".into(),
                    title: None,
                    description: Some("Complete handshake example".into()),
                    mime_type: Some("text/x-rust".into()),
                },
                annotations: None,
            },
        ]
    }

    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContents, ErrorData> {
        match uri {
            "zks://examples/basic_connection" => {
                let content = self.generate_basic_connection_example();
                Ok(ResourceContents::TextResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/x-rust".to_string()),
                    text: content,
                    meta: None,
                })
            }
            "zks://examples/anonymous_connection" => {
                let content = self.generate_anonymous_connection_example();
                Ok(ResourceContents::TextResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/x-rust".to_string()),
                    text: content,
                    meta: None,
                })
            }
            "zks://examples/file_transfer" => {
                let content = self.generate_file_transfer_example();
                Ok(ResourceContents::TextResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/x-rust".to_string()),
                    text: content,
                    meta: None,
                })
            }
            "zks://examples/keypair_generation" => {
                let content = self.generate_keypair_example();
                Ok(ResourceContents::TextResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/x-rust".to_string()),
                    text: content,
                    meta: None,
                })
            }
            "zks://examples/handshake" => {
                let content = self.generate_handshake_example();
                Ok(ResourceContents::TextResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/x-rust".to_string()),
                    text: content,
                    meta: None,
                })
            }
            _ => Err(rmcp::ErrorData::resource_not_found(format!("Unknown example resource: {}", uri), None))
        }
    }

    fn generate_basic_connection_example(&self) -> String {
        r#"use zks_sdk::builder::ZkConnectionBuilder;
use zks_sdk::protocol::ZkUrl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the ZKS URL
    let url = ZkUrl::parse("zk://example.com:8080")?;
    
    // Build a direct connection
    let mut connection = ZkConnectionBuilder::new()
        .url(url)
        .security("post-quantum")
        .timeout(30)
        .build()?;
    
    // Establish connection
    connection.connect().await?;
    println!("Connected to {}", connection.peer_info().address);
    
    // Send a message
    let message = b"Hello, ZKS!";
    connection.send(message).await?;
    
    // Receive response
    let response = connection.receive().await?;
    println!("Received: {:?}", response);
    
    // Close connection
    connection.close().await?;
    println!("Connection closed");
    
    Ok(())
}"#.to_string()
    }

    fn generate_anonymous_connection_example(&self) -> String {
        r#"use zks_sdk::builder::ZksConnectionBuilder;
use zks_sdk::protocol::ZkUrl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the ZKS swarm URL
    let url = ZkUrl::parse("zks://swarm.example.com:8080")?;
    
    // Build an anonymous connection with onion routing
    let mut connection = ZksConnectionBuilder::new()
        .url(url)
        .min_hops(3)
        .max_hops(5)
        .enable_scrambling(true)
        .security("post-quantum")
        .build()?;
    
    // Establish anonymous connection
    connection.connect().await?;
    println!("Anonymous connection established");
    println!("Route length: {} hops", connection.route_info().hops);
    
    // Send anonymous message
    let message = b"Anonymous message";
    connection.send(message).await?;
    
    // Receive anonymous response
    let response = connection.receive().await?;
    println!("Anonymous response: {:?}", response);
    
    // Close anonymous connection
    connection.close().await?;
    println!("Anonymous connection closed");
    
    Ok(())
}"#.to_string()
    }

    fn generate_file_transfer_example(&self) -> String {
        r#"use zks_sdk::builder::ZkConnectionBuilder;
use zks_sdk::protocol::ZkUrl;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read file to transfer
    let file_data = fs::read("document.pdf")?;
    
    // Parse the ZKS URL
    let url = ZkUrl::parse("zk://secure.example.com:8080")?;
    
    // Build secure connection
    let mut connection = ZkConnectionBuilder::new()
        .url(url)
        .security("post-quantum")
        .timeout(60)
        .build()?;
    
    // Establish connection
    connection.connect().await?;
    println!("Connected to {}", connection.peer_info().address);
    
    // Send file metadata
    let metadata = serde_json::json!({
        "filename": "document.pdf",
        "size": file_data.len(),
        "type": "application/pdf"
    });
    connection.send(metadata.to_string().as_bytes()).await?;
    
    // Send file data in chunks
    let chunk_size = 1024;
    for chunk in file_data.chunks(chunk_size) {
        connection.send(chunk).await?;
    }
    
    // Send completion signal
    connection.send(b"FILE_TRANSFER_COMPLETE").await?;
    
    // Wait for acknowledgment
    let ack = connection.receive().await?;
    println!("Transfer acknowledgment: {:?}", ack);
    
    // Close connection
    connection.close().await?;
    println!("File transfer completed");
    
    Ok(())
}"#.to_string()
    }

    fn generate_keypair_example(&self) -> String {
        r#"use zks_sdk::crypto::ml_kem;
use zks_sdk::crypto::ml_dsa;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate ML-KEM-768 keypair for encryption
    println!("Generating ML-KEM-768 keypair...");
    let (pk_enc, sk_enc) = ml_kem::generate_keypair_768()?;
    
    println!("Public key (encryption): {} bytes", pk_enc.len());
    println!("Private key (encryption): {} bytes", sk_enc.len());
    
    // Generate ML-DSA-65 keypair for signatures
    println!("\\nGenerating ML-DSA-65 keypair...");
    let (vk_sig, sk_sig) = ml_dsa::generate_keypair_65()?;
    
    println!("Verifying key (signatures): {} bytes", vk_sig.len());
    println!("Signing key (signatures): {} bytes", sk_sig.len());
    
    // Save keys to files
    std::fs::write("ml_kem_768_pk.bin", &pk_enc)?;
    std::fs::write("ml_kem_768_sk.bin", &sk_enc)?;
    std::fs::write("ml_dsa_65_vk.bin", &vk_sig)?;
    std::fs::write("ml_dsa_65_sk.bin", &sk_sig)?;
    
    println!("\\nKeys saved to files:");
    println!("- ml_kem_768_pk.bin (encryption public key)");
    println!("- ml_kem_768_sk.bin (encryption private key)");
    println!("- ml_dsa_65_vk.bin (signature verifying key)");
    println!("- ml_dsa_65_sk.bin (signature signing key)");
    
    println!("\\nPost-quantum keypairs generated successfully!");
    
    Ok(())
}"#.to_string()
    }

    fn generate_handshake_example(&self) -> String {
        r#"use zks_sdk::protocol::{Handshake, HandshakeRole};
use zks_sdk::crypto::ml_kem;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate keypairs for both parties
    let (alice_pk, alice_sk) = ml_kem::generate_keypair_768()?;
    let (bob_pk, bob_sk) = ml_kem::generate_keypair_768()?;
    
    // Alice initiates handshake
    println!("Alice initiating handshake...");
    let mut alice_handshake = Handshake::new_initiator(alice_sk.clone())?;
    
    // Alice creates first message
    let init_message = alice_handshake.create_init(bob_pk.clone())?;
    println!("Alice -> Bob: Initiation message ({} bytes)", init_message.len());
    
    // Bob receives and processes first message
    println!("\\nBob processing initiation...");
    let mut bob_handshake = Handshake::new_responder(bob_sk.clone())?;
    let response = bob_handshake.process_init(init_message)?;
    
    // Bob creates response
    let response_message = bob_handshake.create_response(alice_pk.clone())?;
    println!("Bob -> Alice: Response message ({} bytes)", response_message.len());
    
    // Alice receives and processes response
    println!("\\nAlice processing response...");
    alice_handshake.process_response(response_message)?;
    
    // Both parties derive shared secrets
    let alice_secret = alice_handshake.derive_secret()?;
    let bob_secret = bob_handshake.derive_secret()?;
    
    println!("\\nHandshake completed!");
    println!("Alice's shared secret: {} bytes", alice_secret.len());
    println!("Bob's shared secret: {} bytes", bob_secret.len());
    
    // Verify secrets match
    if alice_secret == bob_secret {
        println!("✓ Shared secrets match!");
        println!("✓ Post-quantum secure channel established");
    } else {
        println!("✗ Shared secrets do not match!");
        return Err("Handshake failed".into());
    }
    
    Ok(())
}"#.to_string()
    }
}

impl Default for ExamplesResource {
    fn default() -> Self {
        Self::new(".".to_string())
    }
}