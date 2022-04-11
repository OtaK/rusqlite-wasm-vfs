use crate::{WasiVFSError, WasiVFSResult};
use libsqlite3_sys::{sqlite3_file, sqlite3_file_sqlite3_io_methods};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{IdbObjectStore, IdbTransaction};

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
    sqlite3fs_methods: sqlite3_file_sqlite3_io_methods,
    sqlite3fs: sqlite3_file,
    idb: std::sync::Arc<std::sync::RwLock<Option<web_sys::IdbDatabase>>>,
}

impl IndexedDBFile {
    pub async fn open<S: AsRef<str>>(filename: S) -> WasiVFSResult<Self> {
        let window = web_sys::window().ok_or(WasiVFSError::NoSupport)?;
        let idb_factory = window
            .indexed_db()
            .map_err(|e| WasiVFSError::WebError(e.into_serde().unwrap()))?
            .ok_or(WasiVFSError::NoSupport)?;

        let idb = std::sync::Arc::new(std::sync::RwLock::new(None));

        let cvar = std::sync::Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));
        let cvar_inner = std::sync::Arc::clone(&cvar);

        let idb_open_req = idb_factory
            .open(filename.as_ref())
            .map_err(|e| WasiVFSError::WebError(e.into_serde().unwrap()))?;

        let on_success = Closure::once(Box::new(move |_event: web_sys::Event| {
            let (lock, cvar) = &*cvar_inner;
            *lock.lock().unwrap() = true;
            cvar.notify_one();
        }));
        let on_upgrade = Closure::once(Box::new(move |event: web_sys::Event| {
            let db: web_sys::IdbDatabase = event.target().unwrap().unchecked_into();
            if !db.object_store_names().contains("data") {
                db.create_object_store("data").unwrap();
            }
        }));

        idb_open_req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        idb_open_req.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));

        let (lock, cvar) = &*cvar;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }

        let db: web_sys::IdbDatabase = idb_open_req.result().unwrap().unchecked_into();
        *idb.write().unwrap() = Some(db);

        // TODO: Maybe do this outside of this file? No idea
        let sqlite3fs_methods = sqlite3_file_sqlite3_io_methods {
            iVersion: 1,
            xRead: todo!(),
            xClose: todo!(),
            xWrite: todo!(),
            xTruncate: todo!(),
            xSync: todo!(),
            xFileSize: todo!(),
            xLock: todo!(),
            xUnlock: todo!(),
            xCheckReservedLock: todo!(),
            xFileControl: todo!(),
            xSectorSize: todo!(),
            xDeviceCharacteristics: todo!(),
        };

        Ok(IndexedDBOpenFuture(Self {
            sqlite3fs: sqlite3_file {
                pMethods: &sqlite3fs_methods,
            },
            sqlite3fs_methods,
            idb,
        })
        .await?)
    }

    #[inline]
    fn get_store<S: AsRef<str>>(
        &self,
        store: S,
    ) -> WasiVFSResult<(IdbObjectStore, IdbTransaction)> {
        let transaction = self
            .idb
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .transaction_with_str(store.as_ref())
            .unwrap();

        let store = transaction.object_store(store.as_ref()).unwrap();

        Ok((store, transaction))
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
