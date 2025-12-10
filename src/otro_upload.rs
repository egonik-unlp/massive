const BUCKET_NAME: &str = "salameconqueso";

// const COLLECTION_NAME: &str = "ingredientes_vec2";
const COLLECTION_NAME: &str = "openai_text-embedding-3-small_Dot_dataset_gv_procesados";
const VECTOR_DB_URL: &str =
    "https://d509ad12-5e96-47be-9b89-1cacd36ad567.us-west-1-0.aws.cloud.qdrant.io";

use crate::components::upload_and_display::{ImageInference, Ingredients, Response};
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
            println!("Path = {}", path.display());
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
    let begin = std::time::Instant::now();
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
            println!("Path = {}", path.display());
            let end = std::time::Instant::now() - begin;
            log!(
                "Tardo {} s en enviarse la imagen al servikor aproximandamente",
                end.as_secs_f64()
            );
            return Ok(path.display().to_string());
        }
    }

    Err(ServerFnError::Args("no `file` field found".into()))
}

#[server]
pub async fn upload_to_bucket(server_filepath: String) -> Result<String, ServerFnError> {
    use crate::storage::bucket::BucketStorage;
    let client = BucketStorage::new()
        .await
        .map_err(|err| ServerFnError::new(format!("{err:#?}")))?;
    let remote_url = client
        .upload_file(BUCKET_NAME, &server_filepath)
        .await
        .map_err(|err| ServerFnError::new(format!("{err:#?}")))?;
    Ok(remote_url)
}
// TODO: Pensar si no tiene sentido tener un estado a traves de la app que tenga info del modelo
// usado para el embedding. También ver de otras variables comunes. Podrian estar los conectores a
// los servicios que estoy usando.
#[server]
pub async fn search_in_vector_database(
    inference: ImageInference,
) -> Result<Vec<Ingredients>, ServerFnError> {
    use crate::db::vector_database;
    use crate::db::vector_database::{ConnectedDB, Location, QdrantDatabase};
    use crate::openai::{self, server::model_server::Model};
    use qdrant_client::qdrant::Distance::Cosine;
    let location =
        Location::new_remote(VECTOR_DB_URL).map_err(|err| ServerFnError::new(err.to_string()))?;
    let db =
        vector_database::QdrantDatabase::new(location, COLLECTION_NAME.to_string(), Cosine, 384);
    let res = if let Ok(QdrantDatabase::Connected(connected_db)) = db.connect() {
        connected_db
            .search_ingredients_in_db(inference)
            .await
            .map_err(|err| ServerFnError::new(format!("{err:#?}")))?
    } else {
        Default::default()
    };
    Ok(res)
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
