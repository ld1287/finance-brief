//! Host bridge — exposes Rust services to the Splash runtime via
//! `host.call` / `host.fetch`. In production this registers into a `SplashVm`
//! instance, but in this scaffold we only model the surface so the crate
//! stays compilable without pulling the splash runtime as a dependency.

/// Opaque handle to a Splash runtime. Defined here as a stub so signatures are
/// stable; the real type lives in the splash crate.
pub struct SplashVm {
    _private: (),
}

impl SplashVm {
    /// Convenience constructor for scaffolding/tests.
    pub fn dummy() -> Self {
        Self { _private: () }
    }
}

/// Services the Rust layer can serve to Splash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    NewsRefresh,
    QuotesSnapshot,
    QuotesCandles,
    FavsToggle,
    FavsList,
    SettingsLoad,
    SettingsSave,
}

/// A JSON-shaped promise result. For now we model it as a thin wrapper so the
/// scaffold compiles; the real impl will integrate with the splash promise
/// runtime.
#[derive(Debug, Clone)]
pub struct Promise<T> {
    inner: std::marker::PhantomData<T>,
}

impl<T> Promise<T> {
    /// Wrap a synchronous result into a (placeholder) resolved promise.
    pub fn resolved(_value: T) -> Self {
        Self {
            inner: std::marker::PhantomData,
        }
    }
}

/// Register Rust services into a Splash VM. In the scaffold this is a no-op
/// signature — the real wiring lives in the splash runtime crate.
pub fn register(splash: &mut SplashVm) -> Result<(), String> {
    let _ = splash;
    // TODO: bind each variant of `Service` to a JS-callable handler.
    Err("not implemented".into())
}

/// Dispatch a service call. Real logic lands in a follow-up commit.
pub fn handle(_s: Service, _args: serde_json::Value) -> Promise<Result<serde_json::Value, String>> {
    // TODO: route to the right internal function.
    Promise::resolved(Err("not implemented".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn host_signatures_callable() {
        let mut vm = SplashVm::dummy();
        let _ = register(&mut vm);
        let _ = handle(Service::NewsRefresh, json!({}));
        let _ = handle(Service::SettingsLoad, json!({}));
    }
}
