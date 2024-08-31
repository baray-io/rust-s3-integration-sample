use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::Region;

pub async fn get_bucket() -> Bucket {
    let endpoint = std::env::var("S3_ADDERSS").expect("S3_ADDERSS env not found!!!");
    let bucket_name = std::env::var("BUCKET_NAME").expect("BUCKET_NAME env not found!!!");
    let access_key = std::env::var("ACCESS_KEY").expect("ACCESS_KEY env not found!!!");
    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY env not found!!!");

    let region = Region::Custom {
        region: "".to_owned(),
        endpoint: endpoint.to_owned(),
    };

    let credentials =
        Credentials::new(Some(&access_key), Some(&secret_key), None, None, None).unwrap();

    let bucket = Bucket::new(&bucket_name, region, credentials)
        .unwrap()
        .with_path_style();

    bucket
}
