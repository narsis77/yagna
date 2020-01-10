use futures::future::{Either, Future, Map};
use futures::FutureExt;
use std::time::{Duration, Instant};
use tokio::time::{timeout, Elapsed, Timeout};

pub trait IntoDuration {
    fn into_duration(self) -> Duration;
}

impl IntoDuration for u64 {
    #[inline(always)]
    fn into_duration(self) -> Duration {
        Duration::from_millis(self)
    }
}

impl IntoDuration for Duration {
    #[inline(always)]
    fn into_duration(self) -> Duration {
        self
    }
}

macro_rules! impl_into_duration {
    ($ty:ty) => {
        impl IntoDuration for $ty {
            fn into_duration(self) -> Duration {
                if self > 0 as $ty {
                    (self as u64).into_duration()
                } else {
                    Duration::default()
                }
            }
        }
    };
}

impl_into_duration!(i8);
impl_into_duration!(u8);
impl_into_duration!(i16);
impl_into_duration!(u16);
impl_into_duration!(i32);
impl_into_duration!(u32);
impl_into_duration!(i64);
impl_into_duration!(i128);
impl_into_duration!(u128);
impl_into_duration!(f32);
impl_into_duration!(f64);

#[derive(Debug)]
pub struct Interval {
    duration: Duration,
    deadline: Instant,
}

impl Interval {
    pub fn new<D: IntoDuration>(duration: D) -> Self {
        Self {
            duration: duration.into_duration(),
            deadline: Instant::now(),
        }
    }

    #[inline]
    pub fn check(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now >= self.deadline;
        if elapsed {
            self.deadline = now + self.duration;
        }
        elapsed
    }
}

type MapType<F> = Map<F, fn(<F as Future>::Output) -> Result<<F as Future>::Output, Elapsed>>;

pub trait IntoTimeoutFuture<D>: Future
where
    Self: Future + Sized,
    D: IntoDuration,
{
    fn timeout(self, duration: Option<D>) -> Either<Timeout<Self>, MapType<Self>>;
}

impl<F, D> IntoTimeoutFuture<D> for F
where
    Self: Future + Sized,
    D: IntoDuration,
{
    fn timeout(self, duration: Option<D>) -> Either<Timeout<Self>, MapType<Self>> {
        match duration {
            Some(d) => Either::Left(timeout(d.into_duration(), self)),
            None => Either::Right(self.map(|v| Result::Ok(v))),
        }
    }
}
