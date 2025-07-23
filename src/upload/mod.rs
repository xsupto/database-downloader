use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Builder;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{Delete, ObjectCannedAcl, ObjectIdentifier};
use aws_sdk_s3::{Client, Config};
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use log::{error, info};
use std::env;
use std::error::Error;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};
use tokio::fs;
use tokio::io::AsyncReadExt;

struct UploadConfig {
    bucket_name: String,
    region_name: String,
    endpoint_name: String,
    access_key: String,
    secret_key: String,
    path_prefix: String,
    cdn_url: String,
}

impl UploadConfig {
    fn init() -> Result<Self, Box<dyn Error + Send + Sync>> {
        dotenv().ok();
        Ok(Self {
            bucket_name: env::var("DIGITALOCEAN_BUCKET_NAME")?,
            region_name: env::var("DIGITALOCEAN_REGION_NAME")?,
            endpoint_name: env::var("DIGITALOCEAN_ENDPOINT_NAME")?,
            access_key: env::var("DIGITALOCEAN_ACCESS_KEY")?,
            secret_key: env::var("DIGITALOCEAN_SECRET_KEY")?,
            path_prefix: env::var("PATH_PREFIX_CONTENT")
                .unwrap_or_else(|_| "Production".to_string()),
            cdn_url: env::var("DIGITALOCEAN_CDN")?,
        })
    }

    fn client_config(&self) -> Config {
        let credentials =
            Credentials::from_keys(self.access_key.clone(), self.secret_key.clone(), None);
        let config = Builder::new()
            .region(Region::new(self.region_name.clone()))
            .endpoint_url(self.endpoint_name.clone())
            .credentials_provider(credentials)
            .behavior_version_latest()
            .build();

        config
    }

    async fn upload(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let client = Client::from_conf(self.client_config());

        let key = if self.path_prefix != "staging" {
            format!("backup-database/production/{}", file_name)
        } else {
            format!("backup-database/staging/{}", file_name)
        };

        let mut file = fs::File::open(file_name).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let byte_stream = ByteStream::from(buffer);
        let response = client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(byte_stream)
            .content_type("application/octet-stream")
            .acl(ObjectCannedAcl::Private)
            .send()
            .await;

        match response {
            Ok(_) => {
                let url = format!("{}/{}", self.cdn_url, key);
                Self::delete_file(file_name).await?;
                info!("File uploaded successfully! Accessible at: {}", url);
            }
            Err(err) => {
                Self::delete_file(file_name).await?;
                error!("Error uploading file: {}", err);
            }
        }

        Ok(())
    }

    async fn delete_file(file_name: &str) -> Result<(), Box<dyn Error>> {
        if Path::new(file_name).exists() {
            fs::remove_file(file_name).await?;
        }
        Ok(())
    }

    async fn list_and_delete_old_files(&self) -> Result<(), Box<dyn Error>> {
        let client = Client::from_conf(self.client_config());

        let path = if self.path_prefix != "staging" {
            "backup-database/production"
        } else {
            "backup-database/staging"
        };

        let now: DateTime<Utc> = Utc::now();
        let mut objects_to_delete = Vec::new();

        let response = client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .prefix(path)
            .send()
            .await?;

        for object in response.contents() {
            if let Some(last_modified) = object.last_modified() {
                let system_time = UNIX_EPOCH + Duration::from_secs_f64(last_modified.secs() as f64);
                let last_modified_time: DateTime<Utc> = DateTime::<Utc>::from(system_time);
                let retention_period = chrono::Duration::days(4);

                if now.signed_duration_since(last_modified_time) > retention_period {
                    if let Some(key) = object.key() {
                        objects_to_delete.push(ObjectIdentifier::builder().key(key).build()?);
                    }
                }
            }
        }

        if objects_to_delete.is_empty() {
            info!("No old objects found to delete.");
            return Ok(());
        }

        let delete_request = Delete::builder()
            .set_objects(Some(objects_to_delete.clone()))
            .build()?;

        client
            .delete_objects()
            .bucket(&self.bucket_name)
            .delete(delete_request)
            .send()
            .await?;

        info!("Deleted {} old files successfully", objects_to_delete.len());

        Ok(())
    }
}

pub async fn run(file_name: &str) {
    match UploadConfig::init() {
        Ok(config) => {
            if let Err(e) = config.upload(file_name).await {
                error!("Failed to upload file: {}", e);
            }
        }
        Err(e) => error!("Failed to load configuration: {}", e),
    }
}

pub async fn delete_files() {
    match UploadConfig::init() {
        Ok(config) => {
            if let Err(e) = config.list_and_delete_old_files().await {
                error!("Failed to delete files: {}", e);
            }
        }
        Err(e) => error!("Failed to load configuration: {}", e),
    }
}
