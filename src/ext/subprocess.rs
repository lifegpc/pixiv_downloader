use subprocess::{ExitStatus, Popen};

pub struct PopenAwait<'a> {
    pub p: &'a mut Popen,
}

impl<'a> std::future::Future for PopenAwait<'a> {
    type Output = ExitStatus;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.get_mut().p.poll() {
            Some(e) => std::task::Poll::Ready(e),
            None => {
                cx.waker().wake_by_ref();
                return std::task::Poll::Pending;
            }
        }
    }
}

impl<'a> From<&'a mut Popen> for PopenAwait<'a> {
    fn from(value: &'a mut Popen) -> PopenAwait<'a> {
        Self { p: value }
    }
}

pub trait PopenAsyncExt<'a> {
    fn async_wait(&'a mut self) -> PopenAwait<'a>;
}

impl<'a> PopenAsyncExt<'a> for Popen {
    fn async_wait(&'a mut self) -> PopenAwait<'a> {
        PopenAwait::from(self)
    }
}
