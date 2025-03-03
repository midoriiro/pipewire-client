use futures::StreamExt;
use sha2::Digest;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::fs;
use crate::containers::container_api::ContainerApi;

pub(crate) static CONTAINER_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test-utils")
        .join(".containers")
        .as_path()
        .to_path_buf();
    path
});

pub(crate) static CONTAINER_TMP_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = CONTAINER_PATH
        .join(".tmp")
        .as_path()
        .to_path_buf();
    fs::create_dir_all(&path).unwrap();
    path
});

pub(crate) static CONTAINER_DIGESTS_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    CONTAINER_PATH
        .join(".digests")
        .as_path()
        .to_path_buf()
});

pub struct ImageRegistry {
    images: HashMap<String, String>,
}

impl ImageRegistry {
    pub fn new() -> Self {
        let file = match fs::read_to_string(&*CONTAINER_DIGESTS_FILE_PATH) {
            Ok(value) => value,
            Err(_) => return Self {
                images: HashMap::new(),
            }
        };
        let images = file.lines()
            .into_iter()
            .map(|line| {
                let line_parts = line.split("=").collect::<Vec<&str>>();
                let image_name = line_parts[0];
                let container_file_digest = line_parts[1];
                (image_name.to_string(), container_file_digest.to_string().to_string())
            })
            .collect::<HashMap<_, _>>();
        Self {
            images,
        }
    }

    pub fn push(&mut self, image_name: String, container_file_digest: String) {
        if self.images.contains_key(&image_name) {
            *self.images.get_mut(&image_name).unwrap() = container_file_digest
        }
        else {
            self.images.insert(image_name, container_file_digest);
        }
    }
    
    pub fn is_build_needed(&self, image_name: &String, digest: &String) -> bool {
        self.images.get(image_name).map_or(true, |entry| {
            *entry != *digest
        })
    }

    pub(crate) fn cleanup(&self) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&*CONTAINER_DIGESTS_FILE_PATH)
            .unwrap();
        for (image_name, container_file_digest) in self.images.iter() {
            unsafe {
                let format = CString::new("Registering image digests to further process: %s\n").unwrap();
                libc::printf(format.as_ptr() as *const i8, CString::new(image_name.clone()).unwrap()); 
            }
            // println!("Registering image digests to further process: {}", image_name);
            writeln!(
                file,
                "{}={}",
                image_name,
                container_file_digest
            ).unwrap();
        }
    }
}

pub struct ContainerRegistry {
    api: ContainerApi,
    containers: Vec<String>
}

impl ContainerRegistry {
    pub fn new(api: ContainerApi) -> Self {
        let registry = Self {
            api: api.clone(),
            containers: Vec::new(),
        };
        registry.clean();
        registry
    }

    pub(crate) fn clean(&self) {
        let containers = self.api.get_all().unwrap();
        for container in containers {
            let container_id = container.id.unwrap();
            let inspect_result = match self.api.inspect(&container_id) {
                Ok(value) => value,
                Err(_) => continue
            };
            if let Some(state) = inspect_result.state {
                self.api.clean(&container_id.to_string(), &state);
            }
        }
    }
}