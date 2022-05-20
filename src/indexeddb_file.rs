use std::{sync::{Arc, RwLock, Mutex}, task::Waker};

use crate::{WasmVFSError, WasmVFSResult};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{IdbObjectStore, IdbTransaction, IdbCursor};

macro_rules! console_log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into());
    }
}

#[derive(Debug)]
struct IndexedDBOpenFutureState {
    file: IndexedDBFile,
    waker: Option<Waker>,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct IndexedDBOpenFuture(Arc<Mutex<IndexedDBOpenFutureState>>);

impl std::future::Future for IndexedDBOpenFuture {
    type Output = WasmVFSResult<()>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let state_lock = self.0.lock().unwrap();
        let poll_status = match state_lock.file.0.try_read() {
            Ok(maybe_rdb) => {
                console_log!("lock acquired");
                let ret = if maybe_rdb.is_some() {
                    std::task::Poll::Ready(Ok(()))
                } else {
                    std::task::Poll::Pending
                };

                console_log!("Poll status: {ret:?}");

                ret
            }
            Err(_e) => std::task::Poll::Ready(Err(WasmVFSError::PoisonedLock)),
        };

        drop(state_lock);

        match poll_status {
            std::task::Poll::Pending => {
                self.0.lock().unwrap().waker = Some(cx.waker().clone());
            },
            _ => {}
        }

        poll_status
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
#[wasm_bindgen]
pub struct IndexedDBFile {
    db: Arc<RwLock<Option<web_sys::IdbDatabase>>>,
    cursor: Option<IdbCursor>
}

impl IndexedDBFile {
    pub async fn open(filename: impl AsRef<str>) -> WasmVFSResult<Self> {
        let window = web_sys::window().ok_or(WasmVFSError::NoSupport)?;
        let idb_factory = window
            .indexed_db()
            .map_err(|e| WasmVFSError::WebError(e.into_serde().unwrap()))?
            .ok_or(WasmVFSError::NoSupport)?;

        let idb = Arc::new(RwLock::new(None));
        let idb_inner = idb.clone();

        let state: Arc<Mutex<IndexedDBOpenFutureState>> = Mutex::new(IndexedDBOpenFutureState {
            file: Self { db: Arc::clone(&idb), cursor: None },
            waker: None,
        }).into();

        let inner_state = state.clone();

        let idb_open_req = idb_factory
            .open(filename.as_ref())
            .map_err(|e| WasmVFSError::WebError(e.into_serde().unwrap()))?;

        let on_success = Closure::once(Box::new(move |event: web_sys::Event| {
            let target: web_sys::IdbRequest = event.target().unwrap().unchecked_into();
            let db: web_sys::IdbDatabase = target.result().unwrap().unchecked_into();
            *idb_inner.write().unwrap() = Some(db);
            if let Some(waker) = inner_state.lock().unwrap().waker.take() {
                waker.wake();
            }
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

        IndexedDBOpenFuture(state.clone()).await?;

        Ok(Self { db: idb, cursor: None } )
    }

    #[inline]
    fn get_store<S: AsRef<str>>(
        &self,
        store: S,
    ) -> WasmVFSResult<(IdbObjectStore, IdbTransaction)> {
        let transaction = self
            .db
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
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}

impl std::io::Write for IndexedDBFile {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl std::io::Seek for IndexedDBFile {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        // match pos {
        //     std::io::SeekFrom::Start(_) => todo!(),
        //     std::io::SeekFrom::End(_) => todo!(),
        //     std::io::SeekFrom::Current(amount) => {
        //         self
        //     },
        // }
        // if let Some(cursor) = self.cursor {
        //     match pos {
        //         std::io::SeekFrom::Start(_) => todo!(),
        //         std::io::SeekFrom::End(_) => todo!(),
        //         std::io::SeekFrom::Current(amount) => cursor.advance(amount as u32).map(|_| amount).map_err(|_| std::io::Error::from(std::io::ErrorKind::UnexpectedEof)),
        //     }
        // }
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
