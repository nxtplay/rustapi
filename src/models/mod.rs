
// The mod.rs file for the models module

// Declare each of the submodules in the models directory.
// This line tells Rust to look for a file named video.rs or video/mod.rs in the models directory.
pub mod video;

// Optionally, you can re-export items from these submodules.
// This allows other parts of your application to use these items directly
// without needing to know which submodule they come from.
pub use video::Video;

// If you have more structs or enums in separate files, declare and optionally re-export them here.
// For example:
// pub mod user;
// pub use user::User;
