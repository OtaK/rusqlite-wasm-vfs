use crate::{WasiVFSError, WasiVFSResult};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{IdbObjectStore, IdbTransaction};

#[derive(Debug)]
#[repr(transparent)]
pub struct IndexedDBOpenFuture(IndexedDBFile);

impl std::future::Future for IndexedDBOpenFuture {
    type Output = WasiVFSResult<()>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let r_lock = (self.0).0.try_read();
        match r_lock {
            Ok(maybe_rdb) => {
                if maybe_rdb.is_some() {
                    std::task::Poll::Ready(Ok(()))
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
#[wasm_bindgen]
pub struct IndexedDBFile(std::sync::Arc<std::sync::RwLock<Option<web_sys::IdbDatabase>>>);

impl IndexedDBFile {
    pub async fn open<S: AsRef<str>>(filename: S) -> WasiVFSResult<Self> {
        let window = web_sys::window().ok_or(WasiVFSError::NoSupport)?;
        let idb_factory = window
            .indexed_db()
            .map_err(|e| WasiVFSError::WebError(e.into_serde().unwrap()))?
            .ok_or(WasiVFSError::NoSupport)?;

        let idb = std::sync::Arc::new(std::sync::RwLock::new(None));
        let idb_inner = idb.clone();

        let idb_open_req = idb_factory
            .open(filename.as_ref())
            .map_err(|e| WasiVFSError::WebError(e.into_serde().unwrap()))?;

        let on_success = Closure::once(Box::new(move |event: web_sys::Event| {
            let target: web_sys::IdbRequest = event.target().unwrap().unchecked_into();
            let db: web_sys::IdbDatabase = target.result().unwrap().unchecked_into();
            *idb_inner.write().unwrap() = Some(db);
        }));

        let on_upgrade = Closure::once(Box::new(move |event: web_sys::Event| {
            let target: web_sys::IdbRequest = event.target().unwrap().unchecked_into();
            let db: web_sys::IdbDatabase = target.result().unwrap().unchecked_into();
            if !db.object_store_names().contains("data") {
                db.create_object_store("data").unwrap();
            }
        }));

        idb_open_req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        idb_open_req.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));

        IndexedDBOpenFuture(Self(idb.clone())).await?;

        Ok(Self(idb))
    }

    #[inline]
    fn get_store<S: AsRef<str>>(
        &self,
        store: S,
    ) -> WasiVFSResult<(IdbObjectStore, IdbTransaction)> {
        let transaction = self
            .0
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

#[cfg(test)]
mod tests {
    use crate::IndexedDBFile;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn can_open_indexeddb() {
        IndexedDBFile::open("test_file").await.unwrap();
    }
}
