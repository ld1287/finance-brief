//! Desktop entry point.
//!
//! The app itself lives in `lib.rs` because the OpenHarmony build needs a
//! `cdylib`: `app_main!` emits `fn main` only off-mobile, and under
//! `target_env = "ohos"` it emits C entry points that the HAP's native library
//! loads instead. A binary-only crate has nothing for that to live in, so
//! `cargo makepad ohos deveco` fails with "no library targets found".
//!
//! Mirror of `Octoscript-Makepad/apps/flutter-samples/src/main.rs`.
fn main() {
    finance_brief_desktop::app_main()
}
