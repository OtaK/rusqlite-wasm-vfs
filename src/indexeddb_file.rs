use crate::{WasiVFSError, WasiVFSResult};
use libsqlite3_sys::{sqlite3_file, sqlite3_file_sqlite3_io_methods};
use wasm_bindgen::{prelude::*, JsCast};

#[derive(Debug)]
#[repr(transparent)]
pub struct IndexedDBOpenFuture(IndexedDBFile);

impl std::future::Future for IndexedDBOpenFuture {
    type Output = WasiVFSResult<IndexedDBFile>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let r_lock = (self.0).idb.try_read();
        match r_lock {
            Ok(maybe_rdb) => {
                if maybe_rdb.is_some() {
                    std::task::Poll::Ready(Ok(self.0.clone()))
                } else {
                    std::task::Poll::Pending
                }
            }
            Err(_e) => std::task::Poll::Ready(Err(WasiVFSError::PoisonedLock)),
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct IndexedDBFile {
    sqlite3fs: libsqlite3_sys::sqlite3_file,
    idb: std::sync::Arc<std::sync::RwLock<Option<web_sys::IdbDatabase>>>,
}

impl IndexedDBFile {
    pub async fn open<S: AsRef<str>>(filename: S) -> WasiVFSResult<Self> {
        let window = web_sys::window().ok_or(WasiVFSError::NoSupport)?;
        let idb = window
            .indexed_db()
            .map_err(|e| WasiVFSError::WebError(e.into_serde().unwrap()))?
            .ok_or(WasiVFSError::NoSupport)?;

        let inner = std::sync::Arc::new(std::sync::RwLock::new(None));

        let idb_inner = std::sync::Arc::clone(&inner);

        let idb_open_req = idb
            .open(filename.as_ref())
            .map_err(|e| WasiVFSError::WebError(e.into_serde().unwrap()))?;

        let on_success = Closure::once(Box::new(move |event: web_sys::Event| {
            let db: web_sys::IdbDatabase = event.target().unwrap().unchecked_into();
            *idb_inner.write().unwrap() = Some(db);
        }));
        let on_upgrade = Closure::once(Box::new(move |event: web_sys::Event| {
            let db: web_sys::IdbDatabase = event.target().unwrap().unchecked_into();
            if !db.object_store_names().contains("data") {
                db.create_object_store("data").unwrap();
            }
        }));

        idb_open_req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        idb_open_req.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));

        // TODO: Maybe do this outside of this file? No idea
        let sqlite3fs_methods = sqlite3_file_sqlite3_io_methods {
            iVersion: 1,
            xRead: Some(<IndexedDBFile as std::io::Read>::read),
        };

        Ok(IndexedDBOpenFuture(Self {
            sqlite3fs: sqlite3_file {
                pMethods: sqlite3fs_methods,
            },
            idb: inner,
        })
        .await?)
    }
}

impl std::io::Read for IndexedDBFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}

impl std::io::Write for IndexedDBFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl std::io::Seek for IndexedDBFile {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        todo!()
    }
}
