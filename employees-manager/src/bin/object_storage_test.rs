use futures::StreamExt;
use object_store::{gcp::GoogleCloudStorageBuilder, parse_url, ObjectStore};
use url::Url;

#[tokio::main]
async fn main() {
    /*
    let url = Url::parse("s3://bucket/path").unwrap();
    let (store, path) = parse_url(&url).unwrap();
    dbg!(store);
    dbg!(path);
    */
    let url = Url::parse("file:///bucket/path").unwrap();
    let (store, path) = parse_url(&url).unwrap();
    dbg!(store);
    dbg!(path);

    let url = Url::parse("gs://bucket/path").unwrap();
    let (store, path) = parse_url(&url).unwrap();
    dbg!(store);
    dbg!(path);

    google_test().await;
}

async fn google_test() {
    let gcs = GoogleCloudStorageBuilder::from_env()
        .with_bucket_name("ml3-platform-demo-pre")
        .build()
        .unwrap();

    let mut objects = gcs.list(Some(&object_store::path::Path::from(
        "/templates/energy/kpis",
    )));

    while let Some(obj) = objects.next().await.transpose().unwrap() {
        println!("{:?}", obj);
    }

    let result = gcs
        .get(&object_store::path::Path::from(
            "/templates/energy/kpis/net_revenue.parquet",
        ))
        .await
        .unwrap();

    let df_bytes = &result.bytes().await.unwrap();
    let path = std::path::Path::new("lorem_ipsum.parquet");
    let mut file = std::fs::File::create(path).unwrap();

    use std::io::Write;
    file.write_all(df_bytes).unwrap();

    dbg!(df_bytes);
}
