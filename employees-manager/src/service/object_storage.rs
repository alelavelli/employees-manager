use bytes::Bytes;
use object_store::{gcp::GoogleCloudStorageBuilder, local::LocalFileSystem, ObjectStore};

use crate::{enums::ObjectSourceType, error::ServiceAppError};

use super::environment::ENVIRONMENT;

/// Provide utilities and implementation of method to store and load application objects
/// decoupling the services from the actual storage implementation.
/// It is useful because LocalFileSystem does not have signed url and therefore a workaround
/// must be put in place. Moreover, the authentication with cloud providers can be different
/// therefore, using a struct can be the right choice.
///
/// However, it can be changes to simple function if the next developments of the application
/// show that this struct is not necessary.
pub struct ObjectStorageService {
    storage: Box<dyn ObjectStore>,
}

impl ObjectStorageService {
    pub fn new() -> Result<Self, ServiceAppError> {
        let prefix_path = ENVIRONMENT.get_object_storage_prefix_path();
        // as Box<dyn ObjectStore> is required otherwise, the compilers casts the match return type
        // as Box<LocalFileSystem> since it is the first returned one
        let storage: Box<dyn ObjectStore> = match ENVIRONMENT.get_object_storage_source_type() {
            ObjectSourceType::LocalFileSystem => Box::new(
                LocalFileSystem::new_with_prefix(std::path::Path::new(prefix_path)).map_err(
                    |e| {
                        ServiceAppError::ObjectStorageError(format!(
                            "Got error {e} when creating LocalFileSystem storage."
                        ))
                    },
                )?,
            ) as Box<dyn ObjectStore>,

            ObjectSourceType::GcpGS => Box::new(
                GoogleCloudStorageBuilder::from_env()
                    .with_bucket_name(prefix_path)
                    .build()
                    .map_err(|e| {
                        ServiceAppError::ObjectStorageError(format!(
                            "Got error {e} when creating GoogleCloudStorage storage."
                        ))
                    })?,
            ) as Box<dyn ObjectStore>,

            storage_type => Err(ServiceAppError::ObjectStorageError(format!(
                "Unsupported {storage_type} for object storage service"
            )))?,
        };

        Ok(Self { storage })
    }

    /// Load an object from the storage returning it as a sequence of bytes
    ///
    /// Future extensions of this method could cast the bytes into an actual entity
    pub async fn load_object(&self, path: &str) -> Result<Bytes, ServiceAppError> {
        let load_result = self
            .storage
            .get(&object_store::path::Path::from(path))
            .await?;

        Ok(load_result.bytes().await?)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir_in;

    use crate::service::object_storage::ObjectStorageService;

    #[tokio::test]
    async fn local_file_system_test() {
        // Create a directory inside of `env::temp_dir()`.
        let dir = tempdir_in("../app-objects").unwrap();
        let file_path = dir.path().join("my-temporary-note.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello Employees Manager Dev.").unwrap();

        let storage_service = ObjectStorageService::new().unwrap();

        let relative_path = file_path
            .to_str()
            .unwrap()
            .split("app-objects")
            .last()
            .unwrap();

        let result_object = storage_service.load_object(&relative_path).await;
        assert!(result_object.is_ok())
    }
}
