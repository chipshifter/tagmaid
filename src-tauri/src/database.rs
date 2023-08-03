//! The database interface repository.
//! It contains the following elements:
//!
//! High-level components:
//!
//!  - [`tagmaid_database`](tagmaid_database): It is the main and highest level database component.
//! Most useful functions are (and should be) handled there. It acts as the interface
//! for [`TagFile`](crate::data::tag_file) (the files stored in the database). It is thread-safe and based on shared concurrency.
//!
//! Lower-level components:
//!
//!  - [`sqlite_files`](sqlite_files): Used for managing the `_files` table directly. If you want
//! to handle those, using [`TagMaidDatabase`](crate::database::tagmaid_database) is recommended instead.
//!
//!  - [`sqlite_tags`](sqlite_tags): Used for managing the `_tags` table directly. Handling it using
//! [`TagMaidDatabase`](crate::database::tagmaid_database) is recommended instead.
//!
//!  - [`sqlite_taginfo`](sqlite_taginfo): It is the new interface for `_tags`, acting as a replacement for `sqlite_tags`.
//!
//!  - [`filesystem`](filesystem): Used for handling filesystem operations.
pub mod filesystem;
pub mod sqlite_database;
pub mod sqlite_files;
pub mod sqlite_taginfo;
pub mod sqlite_tags;
pub mod tagmaid_database;
