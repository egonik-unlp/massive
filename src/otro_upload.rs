use crate::components::upload_and_display::Response;
use leptos::{logging::log, prelude::*};
use server_fn::codec::{MultipartData, MultipartFormData};
use web_sys::FormData;
// const URL_BACKEND: &'static str = "http://parse_label:8080/";
const URL_BACKEND: &str = "http://35.208.164.235:8010";
const URL_SELF: &str = "http://35.208.164.235:7777";
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub ingredient_vec: Option<Vec<i32>>,
}
#[server(UploadImage, "/api", input = MultipartFormData, endpoint = "rompeme" )]
pub async fn upload_image(data: MultipartData) -> Result<String, ServerFnError> {
    use std::{
        fs::create_dir_all,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    use tokio::io::AsyncWriteExt;

    log!("No si corrio boludo que decis");
    // Ensure the uploads directory exists
    create_dir_all("uploads")
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;

    // Save the first file-like field we get
    let mut mp = match data.into_inner() {
        Some(m) => m,
        None => {
            return Err(ServerFnError::Args(
                "multipart data not available on client".into(),
            ))
        }
    };

    while let Some(field) = mp.next_field().await? {
        // If a specific name is expected, keep this; otherwise accept the first file
        if field.name().unwrap_or("").eq("file") || field.file_name().is_some() {
            let name = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "upload.bin".to_string());
            let bytes = field.bytes().await?;
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let path = PathBuf::from("uploads").join(format!("{ts}_{name}"));

            let mut f = tokio::fs::File::create(&path)
                .await
                .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
            f.write_all(&bytes)
                .await
                .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;

            // Return the relative path; optionally serve it via a static route
            println!("Path = {}", path.display().to_string());
            return Ok(path.display().to_string());
        }
    }

    Err(ServerFnError::Args("no `file` field found".into()))
}

pub async fn transfer_file(files: &Vec<web_sys::File>) -> anyhow::Result<()> {
    let form_data = FormData::new().unwrap();
    for file in files.iter() {
        form_data
            .append_with_blob_and_filename("file", file, file.name().as_str())
            .unwrap();
    }
    anyhow::Ok(())
}

#[server(input = MultipartFormData)]
pub async fn image_upload_to_server(data: MultipartData) -> Result<String, ServerFnError> {
    use image::{DynamicImage, ImageReader};
    use std::io::{Cursor, Read};
    use std::{
        fs::create_dir_all,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    use tokio::io::AsyncWriteExt;
    let mut multipart = data.into_inner();
    log!("No si corrio boludo que decis");
    // Ensure the uploads directory exists
    create_dir_all("uploads")
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    // Save the first file-like field we get
    let mut mp = match multipart {
        Some(m) => {
            println!("m = {:?}", m);
            log!("m = {:?}", m);
            m
        }

        None => {
            return Err(ServerFnError::Args(
                "multipart data not available on client".into(),
            ))
        }
    };

    while let Some(field) = mp.next_field().await? {
        // If a specific name is expected, keep this; otherwise accept the first file
        if field.name().unwrap_or("").eq("file") || field.file_name().is_some() {
            let name = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "upload.bin".to_string());
            let bytes = field.bytes().await?;
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let path = PathBuf::from("uploads").join(format!("{ts}_{name}"));

            let mut f = tokio::fs::File::create(&path)
                .await
                .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;

            f.write_all(&bytes)
                .await
                .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;

            // Return the relative path; optionally serve it via a static route
            log!("Path = {}", path.display().to_string());
            println!("Path = {}", path.display().to_string());
            return Ok(path.display().to_string());
        }
    }

    Err(ServerFnError::Args("no `file` field found".into()))
}

#[server]
pub async fn handle_backend_request(filepath: String) -> Result<Response, ServerFnError> {
    use reqwest::Client;
    // let image_url = format!("http://localhost:8080/{}", filepath);

    let image_url = format!("{}/{}", URL_SELF, filepath);
    log!("Pre request con url {}", image_url);
    let client = Client::new();
    let response = client
        .get(URL_BACKEND)
        .query(&[("image_url", image_url)])
        .send()
        .await?;
    let response_text = response.text().await?;
    log!("response text = {}", response_text);
    let object: Response = serde_json::from_str(&response_text)
        .map_err(|err| ServerFnError::new(format!("Error deserializando el payload {}", err)))?;
    Ok(object)
}
