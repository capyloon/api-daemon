//! Declare a macro for making opaque runtime wrappers.

/// Implement delegating implementations of the runtime traits for a type $t
/// whose member $r implements Runtime.  Used to hide the details of the
/// implementation of $t.
#[allow(unused)] // Can be unused if no runtimes are declared.
macro_rules! implement_opaque_runtime {
{
    $t:ty { $member:ident : $mty:ty }
} => {

    impl futures::task::Spawn for $t {
        #[inline]
        fn spawn_obj(&self, future: futures::future::FutureObj<'static, ()>) -> Result<(), futures::task::SpawnError> {
            self.$member.spawn_obj(future)
        }
    }

    impl $crate::traits::BlockOn for $t {
        #[inline]
        fn block_on<F: futures::Future>(&self, future: F) -> F::Output {
            self.$member.block_on(future)
        }

    }

    impl $crate::traits::SleepProvider for $t {
        type SleepFuture = <$mty as $crate::traits::SleepProvider>::SleepFuture;
        #[inline]
        fn sleep(&self, duration: std::time::Duration) -> Self::SleepFuture {
            self.$member.sleep(duration)
        }
    }

    #[async_trait::async_trait]
    impl $crate::traits::TcpProvider for $t {
        type TcpStream = <$mty as $crate::traits::TcpProvider>::TcpStream;
        type TcpListener = <$mty as $crate::traits::TcpProvider>::TcpListener;
        #[inline]
        async fn connect(&self, addr: &std::net::SocketAddr) -> std::io::Result<Self::TcpStream> {
            self.$member.connect(addr).await
        }
        #[inline]
        async fn listen(&self, addr: &std::net::SocketAddr) -> std::io::Result<Self::TcpListener> {
            self.$member.listen(addr).await
        }
    }

    impl<S> $crate::traits::TlsProvider<S> for $t
    where S: futures::AsyncRead + futures::AsyncWrite + Unpin + Send + 'static,
    {
        type Connector = <$mty as $crate::traits::TlsProvider<S>>::Connector;
        type TlsStream = <$mty as $crate::traits::TlsProvider<S>>::TlsStream;
        #[inline]
        fn tls_connector(&self) -> Self::Connector {
            self.$member.tls_connector()
        }
    }

    impl std::fmt::Debug for $t {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct(stringify!($t)).finish_non_exhaustive()
        }
    }

    // This boilerplate will fail unless $t implements Runtime.
    const _ : () = {
        fn assert_runtime<R: $crate::Runtime>() {}
        fn check() {
            assert_runtime::<$t>();
        }
    };
}
}

#[allow(unused)] // Can be unused if no runtimes are declared.
pub(crate) use implement_opaque_runtime;
