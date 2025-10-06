use leptos::{logging::log, prelude::*};
use massive::components::upload_and_display::{self, Response};
use server_fn::codec::{MultipartData, MultipartFormData};
/// POST /api/UploadImage (expects multipart/form-data with field name "file")
#[server(UploadImage, "/api", input = MultipartFormData,endpoint = "rompeme" )]
pub async fn upload_image(data: MultipartData) -> Result<String, ServerFnError> {
    use image::{DynamicImage, ImageReader};
    use std::io::{Cursor, Read};
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
        Some(m) => {
            log!(" m = {:? }", m);
            m
        }
        None => {
            return Err(ServerFnError::Args(
                "multipart data not available on client".into(),
            ))
        }
    };

    log!("Distinto aca ");
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
            let image = ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()?
                .decode()?;
            let resized =
                image::imageops::resize(&image, 512, 512, image::imageops::FilterType::Nearest)
                    .into_raw();
            f.write_all(&resized)
                .await
                .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;

            println!("Distinto aca {}", path.display());
            log!("Distinto aca {}", path.display().to_string());
            return Ok(path.display().to_string());
        }
    }

    Err(ServerFnError::Args("no `file` field found".into()))
}
