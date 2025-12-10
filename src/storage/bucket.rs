use std::fmt::format;

use anyhow::{Context, Ok};
use bytes::Bytes;
use google_cloud_storage::{
    client::{Storage, StorageControl},
    model::Bucket,
};

use crate::storage::compressor::Compressor;

#[allow(dead_code)]
#[derive(Debug)]
pub struct BucketStorage {
    pub control: StorageControl,
    pub storage: Storage,
}
impl BucketStorage {
    pub async fn new() -> anyhow::Result<Self> {
        let control = StorageControl::builder().build().await?;
        let storage = Storage::builder().build().await?;
        let st = BucketStorage { control, storage };
        Ok(st)
    }
    pub async fn acquires_bucket(&self, bucket_name: &str) -> anyhow::Result<Bucket> {
        let res = self
            .control
            .get_bucket()
            .set_name(bucket_name)
            .send()
            .await
            .with_context(|| format!("Couldn't acquire bucket {}", bucket_name))?;
        Ok(res)
    }
    pub async fn upload_file(
        &self,
        bucket_name: &str,
        file_location: &str,
    ) -> anyhow::Result<String> {
        let bucket_name_gcp = format!("projects/_/buckets/{bucket_name}");
        let compressor = Compressor::with_sane_defaults();
        let data = tokio::fs::read(file_location)
            .await
            .context("Can't read file")?;
        let compressed_data = compressor
            .compress(data)
            .context("Couldnt compress image data")?;
        let data_bytes = Bytes::from(compressed_data);
        let path = format!("user_uploads/{file_location}");
        let upload = self
            .storage
            .write_object(bucket_name_gcp, path.clone(), data_bytes)
            .send_unbuffered()
            .await
            .map_err(|err| anyhow::Error::msg(err.to_string()))?;
        println!("{:?}", upload);
        let url = format!("https://storage.googleapis.com/{}/{}", bucket_name, path);

        Ok(url)
    }
}
