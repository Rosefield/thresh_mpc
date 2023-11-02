use crate::{
    base_func::{BaseFunc, FuncId},
    party::PartyId,
};

use std::{
    collections::HashMap,
    future::Future,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use tokio::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::Mutex,
};

use log::trace;

pub struct AsyncNetworkMgr<I, O> {
    party_id: PartyId,
    recvs: HashMap<(PartyId, FuncId), Mutex<I>>,
    sends: HashMap<(PartyId, FuncId), Mutex<O>>,
    net_bytes: HashMap<(PartyId, FuncId), AtomicU64>,
}

impl<I, O> BaseFunc for AsyncNetworkMgr<I, O> {
    const FUNC_ID: FuncId = FuncId::Fnet;
    const REQUIRED_FUNCS: &'static [FuncId] = &[];

    fn party(&self) -> PartyId {
        self.party_id
    }
}

/// The trait that represents the network for the protocol.
/// It is responsible for delivering messages to other parties and named sub-components.
///
/// This trait allows for multiple concurrent and parallel sends and receives
pub trait AsyncNet: Send + Sync + 'static {
    /// Sends a message to (`party`, `func`)
    fn send_to<B: AsRef<[u8]> + Send>(
        self: Arc<Self>,
        party: PartyId,
        func: FuncId,
        data: B,
    ) -> impl Future<Output = io::Result<()>> + Send;

    /// Receive a message from (`party`, `func`),
    /// returning the buffer and the number of bytes received if successful.
    fn recv_from(
        self: Arc<Self>,
        party: PartyId,
        func: FuncId,
        buf: Arc<[u8]>,
    ) -> impl Future<Output = io::Result<(Arc<[u8]>, usize)>> + Send;

    /// Sends a message to (`party`, `func`)
    async fn send_to_local<B: AsRef<[u8]>>(
        self: &Self,
        party: PartyId,
        func: FuncId,
        data: B,
    ) -> io::Result<()>;

    /// Receives a message from (`party`, `func`)
    async fn recv_from_local<B: AsMut<[u8]>>(
        self: &Self,
        party: PartyId,
        func: FuncId,
        buf: B,
    ) -> io::Result<(B, usize)>;

    fn reset_stats(self: &Self) -> HashMap<(PartyId, FuncId), u64>;

    /*
    /// Send a message to (`party`, `func`), but multiple bufs
    fn send_to_multi(
        self: Arc<Self>,
        party: PartyId,
        func: FuncId,
        bufs: MultiBuf,
    ) -> impl Future<Output = io::Result<MultiBuf>> + Send;

    /// Receive a message from (`party`, `func`), but writes into bufs
    fn recv_from_multi(
        self: Arc<Self>,
        party: PartyId,
        func: FuncId,
        bufs: MultiBuf,
    ) -> impl Future<Output = io::Result<(MultiBuf, usize)>> + Send;
    */
}

impl<I: AsyncRead + Unpin + Send + 'static, O: AsyncWrite + Unpin + Send + 'static> AsyncNet
    for AsyncNetworkMgr<I, O>
{
    async fn send_to<B: AsRef<[u8]> + Send>(
        self: Arc<Self>,
        party: PartyId,
        func: FuncId,
        data: B,
    ) -> io::Result<()> {
        self.send_to_local(party, func, data).await
    }

    async fn recv_from(
        self: Arc<Self>,
        party: PartyId,
        func: FuncId,
        mut buf: Arc<[u8]>,
    ) -> io::Result<(Arc<[u8]>, usize)> {
        let b = Arc::get_mut(&mut buf).unwrap();

        let (_, s) = self.recv_from_local(party, func, b).await?;

        Ok((buf, s))
    }

    async fn send_to_local<B: AsRef<[u8]>>(
        self: &Self,
        party: PartyId,
        func: FuncId,
        data: B,
    ) -> io::Result<()> {
        let data = data.as_ref();

        trace!(
            "{}: send to ({:?}, {}) size {}",
            self.party_id,
            func,
            party,
            data.len()
        );

        let mut target = self.sends[&(party, func)].lock().await;

        self.net_bytes[&(party, func)].fetch_add(data.len() as u64, Ordering::SeqCst);

        let _ = target.write(&(data.len() as u32).to_le_bytes()).await?;
        let _ = target.write(data).await?;
        target.flush().await?;

        Ok(())
    }

    async fn recv_from_local<B: AsMut<[u8]>>(
        self: &Self,
        party: PartyId,
        func: FuncId,
        mut buf: B,
    ) -> io::Result<(B, usize)> {
        let mut other = self.recvs[&(party, func)].lock().await;

        let mut lb = [0u8; 4];
        other.read(&mut lb).await?;
        let size: usize = u32::from_le_bytes(lb).try_into().unwrap();

        let b = buf.as_mut();
        trace!(
            "{}: recv from ({:?}, {}), size {}/ buf {}",
            self.party_id,
            func,
            party,
            size,
            b.len(),
        );

        // for now
        assert!(
            size <= b.len(),
            "self = {}, other = {party}, func = {func:?}, size = {size}, buf = {}",
            self.party_id,
            b.len()
        );

        self.net_bytes[&(party, func)].fetch_add(size as u64, Ordering::SeqCst);

        other.read_exact(&mut b[..size]).await?;

        Ok((buf, size))
    }
    fn reset_stats(self: &Self) -> HashMap<(PartyId, FuncId), u64> {
        // retrieve and reset stats
        self.net_bytes
            .iter()
            .map(|(k, v)| (k.clone(), v.swap(0, Ordering::SeqCst)))
            .collect()
    }

    /*
    /// Send a message to (`party`, `func`), but with multiple bufs
    async fn send_to_multi(
        self: Arc<Self>,
        party: PartyId,
        func: FuncId,
        bufs: MultiBuf,
    ) -> io::Result<MultiBuf> {
        let mut target = self.sends[&(party, func)].lock().await;

        let total_size: usize = bufs.total_size();
        let _ = target.write(&(total_size as u32).to_le_bytes()).await?;
        for b in bufs.iter() {
            let _ = target.write(b).await?;
        }
        target.flush().await?;

        Ok(bufs)
    }

    /// Receive a message from (`party`, `func`), but writes into bufs if necessary
    async fn recv_from_multi(
        self: Arc<Self>,
        party: PartyId,
        func: FuncId,
        mut bufs: MultiBuf,
    ) -> io::Result<(MultiBuf, usize)> {
        let mut other = self.recvs[&(party, func)].lock().await;

        let mut lb = [0u8; 4];
        other.read(&mut lb).await?;
        let size: usize = u32::from_le_bytes(lb).try_into().unwrap();

        // for now
        let avail_size = bufs.total_size();
        assert!(size <= avail_size);

        let mut remaining = size;
        while remaining > 0 {
            let b = bufs.next_buf_mut().unwrap();
            // fill each of the bufs in order, with the last buf partially filled
            let r = std::cmp::min(b.len(), remaining);
            other.read_exact(&mut b[..r]).await?;
            remaining -= r;
        }

        Ok((bufs, size))
    }
    */
}

impl<I: AsyncRead, O: AsyncWrite> AsyncNetworkMgr<I, O> {
    pub fn new(
        party_id: PartyId,
        _num_parties: usize,
        senders: HashMap<(PartyId, FuncId), O>,
        receivers: HashMap<(PartyId, FuncId), I>,
    ) -> Result<Self, ()> {
        let net_bytes = senders
            .keys()
            .map(|k| (k.clone(), AtomicU64::new(0)))
            .collect();

        Ok(AsyncNetworkMgr {
            party_id: party_id,
            sends: senders
                .into_iter()
                .map(|(k, v)| (k, Mutex::new(v)))
                .collect(),
            recvs: receivers
                .into_iter()
                .map(|(k, v)| (k, Mutex::new(v)))
                .collect(),
            net_bytes: net_bytes,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::party::{PartyId, PartyInfo};
    use std::{net::IpAddr, str::FromStr};
    use tokio::net::{TcpListener, TcpStream};

    pub fn get_test_party_infos(num: PartyId) -> Vec<PartyInfo> {
        (1..=num)
            .map(|p| PartyInfo {
                id: p,
                ip: IpAddr::from_str(&format!("127.0.0.1")).unwrap(),
                port: 9000 + 1000 * p,
            })
            .collect()
    }

    pub async fn build_test_nets(
        party_info: &[PartyInfo],
        funcs: Vec<FuncId>,
    ) -> Vec<Arc<AsyncNetworkMgr<TcpStream, TcpStream>>> {
        let mut nets = Vec::new();

        let mut senders: HashMap<PartyId, HashMap<(PartyId, FuncId), TcpStream>> = HashMap::new();
        let mut receivers: HashMap<PartyId, HashMap<(PartyId, FuncId), TcpStream>> = HashMap::new();

        for pi in party_info.iter() {
            for pj in party_info.iter().filter(|pj| pj.id != pi.id) {
                for &f in funcs.iter() {
                    // ask for a new port
                    let address = (pi.ip.clone(), 0);
                    let listener = TcpListener::bind(address.clone()).await.unwrap();
                    let actual_address = listener.local_addr().unwrap();
                    let hs = tokio::spawn(async move { listener.accept().await.unwrap() });
                    let hr =
                        tokio::spawn(
                            async move { TcpStream::connect(actual_address).await.unwrap() },
                        );

                    let (s, _) = hs.await.unwrap();
                    let r = hr.await.unwrap();

                    senders
                        .entry(pi.id)
                        .or_insert(HashMap::new())
                        .insert((pj.id, f), s);
                    receivers
                        .entry(pj.id)
                        .or_insert(HashMap::new())
                        .insert((pi.id, f), r);
                }
            }
        }

        for pi in party_info.iter() {
            let n = AsyncNetworkMgr::new(
                pi.id,
                party_info.len(),
                senders.remove(&pi.id).unwrap(),
                receivers.remove(&pi.id).unwrap(),
            );
            nets.push(Arc::new(n.unwrap()));
        }

        nets
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_recv() -> io::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:5000").await?;

        let h1 = tokio::spawn(async move {
            let (s, _) = listener.accept().await.unwrap();
            let mut sends = HashMap::new();
            sends.insert((2, FuncId::Ftest), s);
            let net1: Arc<AsyncNetworkMgr<TcpStream, TcpStream>> =
                Arc::new(AsyncNetworkMgr::new(1, 2, sends, HashMap::new()).unwrap());
            let r = net1
                .send_to(2, FuncId::Ftest, Arc::from([1, 2, 3, 4].as_slice()))
                .await;
            assert!(r.is_ok());
        });

        let h2 = tokio::spawn(async {
            let s = TcpStream::connect("127.0.0.1:5000").await.unwrap();
            let mut recvs = HashMap::new();
            recvs.insert((1, FuncId::Ftest), s);
            let net2: Arc<AsyncNetworkMgr<TcpStream, TcpStream>> =
                Arc::new(AsyncNetworkMgr::new(2, 2, HashMap::new(), recvs).unwrap());
            let buf = Arc::from([0; 4]);
            let r = net2.recv_from(1, FuncId::Ftest, buf).await;
            assert!(r.is_ok());
            let (b, _) = r.unwrap();
            assert!(*b == [1, 2, 3, 4]);
        });

        h1.await?;
        h2.await?;

        Ok(())
    }
}
