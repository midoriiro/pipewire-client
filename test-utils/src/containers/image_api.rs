use bollard::errors::Error;
use bollard::image::{BuildImageOptions, BuilderVersion};
use bollard::models::ImageInspect;
use bollard::Docker;
use bytes::Bytes;
use futures::StreamExt;
use pipewire_common::utils::HexSlice;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};
use tar::{Builder, Header};
use tokio::runtime::Runtime;
use uuid::Uuid;

struct ImageContext;

impl ImageContext {
    fn create(container_file_path: &PathBuf) -> Result<(Bytes, String), Error> {
        let excluded_filename = vec![
            ".digests",
        ];
        let context_path = container_file_path.parent().unwrap();
        // Hasher is used for computing all context files hashes.
        // In that way we can determine later with we build the image or not.
        // This is better that just computing context archive hash which include data and metadata
        // that can change regarding if context files had not changed in times.
        let mut hasher = Sha256::new();
        let mut archive = tar::Builder::new(Vec::new());
        Self::read_directory(
            &mut archive,
            &mut hasher,
            context_path,
            context_path,
            Some(&excluded_filename)
        )?;
        let uncompressed = archive.into_inner()?;
        let mut compressed = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        compressed.write_all(&uncompressed)?;
        let compressed = compressed.finish()?;
        let data = Bytes::from(compressed);
        let hash_bytes = hasher.finalize().to_vec();
        let digest = HexSlice(hash_bytes.as_slice());
        let digest = format!("sha256:{}", digest);
        Ok((data, digest))
    }

    fn read_directory(
        archive: &mut Builder<Vec<u8>>,
        hasher: &mut impl Write,
        root: &Path,
        directory: &Path,
        excluded_filenames: Option<&Vec<&str>>
    ) -> io::Result<()> {
        if directory.is_dir() {
            for entry in fs::read_dir(directory)? {
                let entry = entry?;
                let path = entry.path();
                let filename = path.file_name().unwrap().to_str().unwrap();
                if path.is_dir() {
                    Self::read_directory(archive, hasher, root, &path, excluded_filenames)?;
                }
                else if path.is_file() {
                    if excluded_filenames.as_ref().unwrap().contains(&filename) {
                        continue;
                    }
                    let mut file = File::open(&path)?;
                    io::copy(&mut file, hasher)?;
                    file.seek(SeekFrom::Start(0))?;
                    let mut header = Header::new_gnu();
                    let metadata = file.metadata()?;
                    header.set_path(path.strip_prefix(root).unwrap())?;
                    header.set_size(metadata.len());
                    header.set_mode(metadata.permissions().mode());
                    header.set_mtime(metadata.modified()?.elapsed().unwrap().as_secs());
                    header.set_cksum();
                    archive.append(&header, &mut file)?;
                }
            }
        }
        Ok(())
    }
}

pub struct ImageApi {
    runtime: Arc<Runtime>,
    api: Arc<Docker>
}

impl ImageApi {
    pub fn new(runtime: Arc<Runtime>, api: Arc<Docker>) -> Self {
        Self {
            runtime,
            api,
        }
    }

    pub fn inspect(&self, image_name: &String) -> Result<ImageInspect, Error> {
        let result = self.api.inspect_image(image_name.as_str());
        self.runtime.block_on(result)
    }

    pub fn context_build(&self, container_file_path: &PathBuf) -> (Bytes, String) {
        ImageContext::create(&container_file_path).unwrap()
    }

    pub fn build(
        &self,
        container_file_path: &PathBuf,
        image_name: &String,
        image_tag: &String
    ) {
        let tag = format!("{}:{}", image_name, image_tag);
        let options = BuildImageOptions {
            dockerfile: container_file_path.file_name().unwrap().to_str().unwrap(),
            t: tag.as_str(),
            session: Some(Uuid::new_v4().to_string()),
            version: BuilderVersion::BuilderBuildKit,
            ..Default::default()
        };
        let (context, context_digest) = ImageContext::create(&container_file_path).unwrap();
        // TODO 
        // let mut environment = TEST_ENVIRONMENT.lock().unwrap();
        // println!("Container image digest: {}", context_digest);
        // if environment.container_image_registry.is_build_needed(&image_name, &context_digest) == false {
        //     println!("Skip build container image: {}", tag);
        //     return;
        // }
        println!("Build container image: {}", tag);
        let mut stream = self.api.build_image(options, None, Some(context));
        while let Some(message) = self.runtime.block_on(stream.next()) {
            match message {
                Ok(message) => {
                    if let Some(stream) = message.stream {
                        if cfg!(debug_assertions) {
                            print!("{}", stream)
                        }
                    }
                    else if let Some(error) = message.error {
                        panic!("{}", error);
                    }
                }
                Err(value) => {
                    panic!("Error during image build: {:?}", value);
                }
            }
        };
        // TODO
        // environment.container_image_registry.push(
        //     image_name.clone(),
        //     context_digest.clone()
        // );
    }
}

impl Clone for ImageApi {
    fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            api: self.api.clone(),
        }
    }
}