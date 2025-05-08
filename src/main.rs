use clap::{Arg, Command};
use reqwest::multipart::{Form, Part};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("File Sharer")
        .version("1.0")
        .about("Gera um link simplificado para compartilhamento de arquivos")
        .arg(
            Arg::new("file_path")
                .required(true)
                .help("Caminho do arquivo para compartilhar"),
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file_path").unwrap();
    let path = Path::new(file_path);

    if !path.exists() || !path.is_file() {
        eprintln!("Erro: O arquivo '{}' não existe ou não é um arquivo válido.", file_path);
        std::process::exit(1);
    }

    let file_name = path.file_name().unwrap().to_str().unwrap();
    let mut file = File::open(path)?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)?;

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36")
        .build()?;
    let part = Part::bytes(file_content).file_name(file_name.to_string());
    let form = Form::new().part("file", part);

    let upload_response = client
        .post("https://tmpfiles.org/api/v1/upload")
        .multipart(form)
        .send()
        .await?;

    if !upload_response.status().is_success() {
        let status = upload_response.status();
        let response_text = upload_response.text().await?;
        eprintln!("Erro ao fazer upload do arquivo: Status {}, Resposta: {}", status, response_text);
        std::process::exit(1);
    }

    let response_json: serde_json::Value = upload_response.json().await?;
    let download_url = response_json["data"]["url"]
        .as_str()
        .ok_or("Erro: Não foi possível obter o link de download")?
        .to_string();

    let short_url = shorten_url(&client, &download_url).await?;

    println!("Link simplificado para compartilhamento: {}", short_url);
    Ok(())
}

async fn shorten_url(client: &reqwest::Client, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let tinyurl_api = format!("https://tinyurl.com/api-create.php?url={}", url);
    let response = client.get(&tinyurl_api).send().await?;

    if response.status().is_success() {
        Ok(response.text().await?.trim().to_string())
    } else {
        Err(format!("Erro ao encurtar URL: {}", response.status()).into())
    }
}