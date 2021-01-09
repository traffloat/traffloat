use std::future::Future;
use std::time::Duration;

use futures::future::{FutureExt, Map};
use tokio::time as tt;

pub const PING_FREQ: Duration = Duration::from_secs(1);
pub const STD_TIMEOUT: Duration = Duration::from_secs(10);

pub trait TimeoutExt: Future + Sized {
    #[allow(clippy::type_complexity)]
    fn std_timeout(
        self,
    ) -> Map<
        tt::Timeout<Self>,
        fn(
            Result<<Self as Future>::Output, tt::error::Elapsed>,
        ) -> Result<<Self as Future>::Output, anyhow::Error>,
    > {
        tt::timeout(STD_TIMEOUT, self)
            .map(|result| result.map_err(|_| anyhow::anyhow!("No data received for 10 seconds")))
    }
}

impl<T: Future> TimeoutExt for T {}
