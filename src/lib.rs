mod error;
pub mod indexeddb_file;
// pub mod originfs_file;

pub use self::error::*;

use indexeddb_file::IndexedDBFile;
// use libsqlite3_sys::{sqlite3_file, sqlite3_file_sqlite3_io_methods};

// #[derive(Debug)]
// #[repr(C)]
// pub struct IndexedDBFileVFS {
//     sqlite3fs: sqlite3_file,
//     sqlite3fs_methods: sqlite3_file_sqlite3_io_methods,
//     file: IndexedDBFile,
// }

// impl IndexedDBFileVFS {
//     pub fn new_with_file(file: IndexedDBFile) -> WasiVFSResult<Self> {
//         let sqlite3fs_methods = sqlite3_file_sqlite3_io_methods {
//             iVersion: 1,
//             xRead: todo!(),
//             xClose: todo!(),
//             xWrite: todo!(),
//             xTruncate: todo!(),
//             xSync: todo!(),
//             xFileSize: todo!(),
//             xLock: todo!(),
//             xUnlock: todo!(),
//             xCheckReservedLock: todo!(),
//             xFileControl: todo!(),
//             xSectorSize: todo!(),
//             xDeviceCharacteristics: todo!(),
//         };

//         Ok(Self {
//             sqlite3fs: sqlite3_file {
//                 pMethods: &sqlite3fs_methods,
//             },
//             sqlite3fs_methods,
//             file,
//         })
//     }
// }
