use bytes::Bytes;
use massive::storage::bucket::BucketStorage;
const BUCKET_NAME: &str = "salameconqueso";
const FILENAME: &str = "uploads/1758405727778_istockphoto-1713465965-2048x2048.jpg";

#[tokio::main]
async fn main() {
    let storage = BucketStorage::new().await.unwrap();
    let bucket_name = format!("projects/_/buckets/{BUCKET_NAME}");
    let bucket = storage
        .acquires_bucket(bucket_name.as_str())
        .await
        .expect("Couldn't get bucket");
    let bytes = std::fs::read(FILENAME).unwrap();
    let data = Bytes::from(bytes);
    dbg!(bucket);
    // Formato requerido para el bucket: "projects/_/buckets/<bucket>"
    storage
        .storage
        .write_object(&bucket_name, FILENAME, data)
        .send_unbuffered()
        .await
        .unwrap();
}
