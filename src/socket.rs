use crate::error::{self, handle_result};

use error::SrtError;
use libsrt_sys as srt;
use os_socketaddr::{self, OsSocketAddr};
use srt::sockaddr;

use std::{
    convert::TryInto,
    ffi::c_void,
    iter::FromIterator,
    mem,
    net::{SocketAddr, ToSocketAddrs},
    os::raw::{c_char, c_int},
};

#[cfg(target_family = "unix")]
use libc::linger;

#[cfg(target_os = "windows")]
use winapi::um::winsock2::linger;

type Result<T> = std::result::Result<T, SrtError>;

pub enum SrtSocketStatus {
    Init,
    Opened,
    Listening,
    Connecting,
    Connected,
    Broken,
    Closing,
    Closed,
    NonExist,
}

#[derive(Copy, Clone, Debug)]
pub struct SrtSocket {
    pub id: i32,
}

//General methods
impl SrtSocket {
    pub fn new() -> Result<Self> {
        let result = unsafe { srt::srt_create_socket() };
        if result == -1 {
            error::handle_result(Self { id: 0 }, result)
        } else {
            Ok(Self { id: result })
        }
    }
    pub fn bind<A: ToSocketAddrs>(self, addrs: A) -> Result<Self> {
        if let Ok(addrs) = addrs.to_socket_addrs() {
            for addr in addrs {
                let os_addr: OsSocketAddr = addr.into();
                let result = unsafe {
                    srt::srt_bind(
                        self.id,
                        os_addr.as_ptr() as *const sockaddr,
                        os_addr.len() as i32,
                    )
                };
                return error::handle_result(self, result);
            }
        }
        Err(SrtError::SockFail)
    }
    pub fn rendezvous<A: ToSocketAddrs>(&self, local: A, remote: A) -> Result<()> {
        let local_addr;
        if let Ok(mut addr) = local.to_socket_addrs() {
            local_addr = addr.next()
        } else {
            return Err(SrtError::SockFail);
        };
        let remote_addr;
        if let Ok(mut addr) = remote.to_socket_addrs() {
            remote_addr = addr.next()
        } else {
            return Err(SrtError::SockFail);
        };

        if let (Some(local), Some(remote)) = (local_addr, remote_addr) {
            let os_local: OsSocketAddr = local.into();
            let os_remote: OsSocketAddr = remote.into();
            let result = unsafe {
                srt::srt_rendezvous(
                    self.id,
                    os_local.as_ptr() as *const sockaddr,
                    os_local.len() as i32,
                    os_remote.as_ptr() as *const sockaddr,
                    os_remote.len() as i32,
                )
            };
            error::handle_result((), result)
        } else {
            Err(SrtError::SockFail)
        }
    }
    pub fn connect<A: ToSocketAddrs>(&self, addrs: A) -> Result<()> {
        let target_addr: SocketAddr;
        if let Ok(mut target) = addrs.to_socket_addrs() {
            if let Some(addr) = target.next() {
                target_addr = addr;
            } else {
                return Err(SrtError::SockFail);
            }
        } else {
            return Err(SrtError::SockFail);
        };
        let os_target: OsSocketAddr = target_addr.into();
        let result = unsafe {
            srt::srt_connect(
                self.id,
                os_target.as_ptr() as *const sockaddr,
                os_target.len() as i32,
            )
        };
        return error::handle_result((), result);
    }
    pub fn listen(&self, backlog: i32) -> Result<()> {
        let result = unsafe { srt::srt_listen(self.id, backlog) };
        error::handle_result((), result)
    }
    pub fn bistats(&self) -> Result<srt::SRT_TRACEBSTATS> {
        let mut stats = srt::SRT_TRACEBSTATS {
            msTimeStamp: 0,
            pktSentTotal: 0,
            pktRecvTotal: 0,
            pktSndLossTotal: 0,
            pktRcvLossTotal: 0,
            pktRetransTotal: 0,
            pktSentACKTotal: 0,
            pktRecvACKTotal: 0,
            pktSentNAKTotal: 0,
            pktRecvNAKTotal: 0,
            usSndDurationTotal: 0,
            pktSndDropTotal: 0,
            pktRcvDropTotal: 0,
            pktRcvUndecryptTotal: 0,
            byteSentTotal: 0,
            byteRecvTotal: 0,
            byteRcvLossTotal: 0,
            byteRetransTotal: 0,
            byteSndDropTotal: 0,
            byteRcvDropTotal: 0,
            byteRcvUndecryptTotal: 0,
            pktSent: 0,
            pktRecv: 0,
            pktSndLoss: 0,
            pktRcvLoss: 0,
            pktRetrans: 0,
            pktRcvRetrans: 0,
            pktSentACK: 0,
            pktRecvACK: 0,
            pktSentNAK: 0,
            pktRecvNAK: 0,
            mbpsSendRate: 0.0,
            mbpsRecvRate: 0.0,
            usSndDuration: 0,
            pktReorderDistance: 0,
            pktRcvAvgBelatedTime: 0.0,
            pktRcvBelated: 0,
            pktSndDrop: 0,
            pktRcvDrop: 0,
            pktRcvUndecrypt: 0,
            byteSent: 0,
            byteRecv: 0,
            byteRcvLoss: 0,
            byteRetrans: 0,
            byteSndDrop: 0,
            byteRcvDrop: 0,
            byteRcvUndecrypt: 0,
            usPktSndPeriod: 0.0,
            pktFlowWindow: 0,
            pktCongestionWindow: 0,
            pktFlightSize: 0,
            msRTT: 0.0,
            mbpsBandwidth: 0.0,
            byteAvailSndBuf: 0,
            byteAvailRcvBuf: 0,
            mbpsMaxBW: 0.0,
            byteMSS: 0,
            pktSndBuf: 0,
            byteSndBuf: 0,
            msSndBuf: 0,
            msSndTsbPdDelay: 0,
            pktRcvBuf: 0,
            byteRcvBuf: 0,
            msRcvBuf: 0,
            msRcvTsbPdDelay: 0,
            pktSndFilterExtraTotal: 0,
            pktRcvFilterExtraTotal: 0,
            pktRcvFilterSupplyTotal: 0,
            pktRcvFilterLossTotal: 0,
            pktSndFilterExtra: 0,
            pktRcvFilterExtra: 0,
            pktRcvFilterSupply: 0,
            pktRcvFilterLoss: 0,
            pktReorderTolerance: 0,
            pktSentUniqueTotal: 0,
            pktRecvUniqueTotal: 0,
            byteSentUniqueTotal: 0,
            byteRecvUniqueTotal: 0,
            pktSentUnique: 0,
            pktRecvUnique: 0,
            byteSentUnique: 0,
            byteRecvUnique: 0,
        };
        let result = unsafe {
            srt::srt_bstats(
                self.id,
                &mut stats,
                1
            )
        };
        handle_result(stats, result)
    }
}

//Public operational methods
impl SrtSocket {
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let mut addr = OsSocketAddr::new();
        let mut addrlen: c_int = addr.capacity() as i32;
        let result = unsafe {
            srt::srt_getsockname(
                self.id,
                addr.as_mut_ptr() as *mut sockaddr,
                &mut addrlen as *mut c_int,
            )
        };
        if result == -1 {
            error::handle_result("0.0.0.0:0".parse().unwrap(), result)
        } else {
            error::handle_result(addr.into_addr().unwrap(), 0)
        }
    }
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        let mut addr = OsSocketAddr::new();
        let mut addrlen: c_int = addr.capacity() as i32;
        let result = unsafe {
            srt::srt_getpeername(
                self.id,
                addr.as_mut_ptr() as *mut sockaddr,
                &mut addrlen as *mut c_int,
            )
        };
        if result == -1 {
            error::handle_result("0.0.0.0:0".parse().unwrap(), result)
        } else {
            error::handle_result(addr.into_addr().unwrap(), result)
        }
    }
    pub fn accept(&self) -> Result<(Self, SocketAddr)> {
        let mut addr = OsSocketAddr::new();
        let mut _addrlen: c_int = addr.capacity() as i32;
        let result = unsafe {
            srt::srt_accept(
                self.id,
                addr.as_mut_ptr() as *mut sockaddr,
                &mut _addrlen as *mut c_int,
            )
        };
        if result == -1 {
            error::handle_result((Self { id: 0 }, "0.0.0.0:0".parse().unwrap()), result)
        } else {
            Ok((Self { id: result }, addr.into_addr().unwrap()))
        }
    }
    pub fn close(self) -> Result<()> {
        let result = unsafe { srt::srt_close(self.id) };
        error::handle_result((), result)
    }
    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        let result = unsafe {
            srt::srt_send(
                self.id,
                buf as *const [u8] as *const c_char,
                buf.len() as i32,
            )
        };
        if result == -1 {
            error::handle_result(result as usize, result)
        } else {
            Ok(result as usize)
        }
    }
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let result =
            unsafe { srt::srt_recv(self.id, buf as *mut [u8] as *mut c_char, buf.len() as i32) };
        if result == -1 {
            error::handle_result(result as usize, result)
        } else {
            Ok(result as usize)
        }
    }
    pub fn get_sender_buffer(&self) -> Result<(usize, usize)> {
        let mut blocks = 0;
        let mut bytes = 0;
        let result = unsafe {
            srt::srt_getsndbuffer(self.id, &mut blocks as *mut usize, &mut bytes as *mut usize)
        };
        if result == -1 {
            error::handle_result((blocks, bytes), result)
        } else {
            Ok((blocks, bytes))
        }
    }
    pub fn get_events(&self) -> Result<srt::SRT_EPOLL_OPT> {
        let mut events: i32 = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_EVENT,
                &mut events as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(
            srt::SRT_EPOLL_OPT(events.try_into().expect("invalid events")),
            result,
        )
    }
}
//Public get flag methods
impl SrtSocket {
    pub fn get_flight_flag_size(&self) -> Result<i32> {
        let mut packets = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_FC,
                &mut packets as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(packets, result)
    }
    pub fn get_input_bandwith(&self) -> Result<i64> {
        let mut bytes_per_sec = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_INPUTBW,
                &mut bytes_per_sec as *mut i64 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(bytes_per_sec, result)
    }
    pub fn get_ip_type_of_service(&self) -> Result<i32> {
        let mut tos = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_IPTOS,
                &mut tos as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(tos, result)
    }
    pub fn get_initial_sequence_number(&self) -> Result<i32> {
        let mut sequences = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_ISN,
                &mut sequences as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(sequences, result)
    }
    pub fn get_ip_time_to_live(&self) -> Result<i32> {
        let mut hops = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_IPTTL,
                &mut hops as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(hops, result)
    }
    pub fn get_ipv6_only(&self) -> Result<i32> {
        let mut value = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_IPV6ONLY,
                &mut value as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(value, result)
    }
    pub fn get_km_refresh_rate(&self) -> Result<i32> {
        let mut packets = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_KMREFRESHRATE,
                &mut packets as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(packets, result)
    }
    pub fn get_km_preannounce(&self) -> Result<i32> {
        let mut packets = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_KMPREANNOUNCE,
                &mut packets as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(packets, result)
    }
    pub fn get_linger(&self) -> Result<i32> {
        let mut linger = linger {
            l_onoff: 0,
            l_linger: 0,
        };
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_LINGER,
                &mut linger as *mut linger as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(linger.l_linger as i32, result)
    }
    pub fn get_max_reorder_tolerance(&self) -> Result<i32> {
        let mut packets = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_LOSSMAXTTL,
                &mut packets as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(packets, result)
    }
    pub fn get_max_bandwith(&self) -> Result<i64> {
        let mut bytes_per_sec = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_MAXBW,
                &mut bytes_per_sec as *mut i64 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(bytes_per_sec, result)
    }
    pub fn get_mss(&self) -> Result<i32> {
        let mut bytes = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_MSS,
                &mut bytes as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(bytes, result)
    }
    pub fn get_nak_report(&self) -> Result<bool> {
        let mut enabled = true;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_NAKREPORT,
                &mut enabled as *mut bool as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(enabled, result)
    }
    pub fn get_encryption_key_length(&self) -> Result<i32> {
        let mut len = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_PBKEYLEN,
                &mut len as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(len, result)
    }
    pub fn get_peer_latency(&self) -> Result<i32> {
        let mut msecs = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_PEERLATENCY,
                &mut msecs as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(msecs, result)
    }
    pub fn get_peer_version(&self) -> Result<i32> {
        let mut version = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_PEERVERSION,
                &mut version as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(version, result)
    }
    pub fn get_receive_buffer(&self) -> Result<i32> {
        let mut bytes = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RCVBUF,
                &mut bytes as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(bytes, result)
    }
    pub fn get_receive_data(&self) -> Result<i32> {
        let mut packets = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RCVDATA,
                &mut packets as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(packets, result)
    }
    pub fn get_receive_km_state(&self) -> Result<SrtKmState> {
        let mut state = SrtKmState::Unsecured;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RCVKMSTATE,
                &mut state as *mut SrtKmState as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(state, result)
    }
    pub fn get_receive_latency(&self) -> Result<i32> {
        let mut msecs = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RCVLATENCY,
                &mut msecs as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(msecs, result)
    }
    pub fn get_receive_blocking(&self) -> Result<bool> {
        let mut blocking = true;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RCVSYN,
                &mut blocking as *mut bool as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(blocking, result)
    }
    pub fn get_receive_timeout(&self) -> Result<i32> {
        let mut msecs = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RCVTIMEO,
                &mut msecs as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(msecs, result)
    }
    pub fn get_reject_reason(&self) -> error::SrtRejectReason {
        let result = unsafe { srt::srt_getrejectreason(self.id) };
        srt::SRT_REJECT_REASON(result.try_into().expect("invalid reject code")).into()
    }
    pub fn get_rendezvous(&self) -> Result<bool> {
        let mut rendezvous = false;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RENDEZVOUS,
                &mut rendezvous as *mut bool as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(rendezvous, result)
    }
    pub fn get_reuse_address(&self) -> Result<bool> {
        let mut rendezvous = false;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_REUSEADDR,
                &mut rendezvous as *mut bool as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(rendezvous, result)
    }
    pub fn get_send_buffer(&self) -> Result<i32> {
        let mut bytes = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_SNDBUF,
                &mut bytes as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(bytes, result)
    }
    pub fn get_send_data(&self) -> Result<i32> {
        let mut packets = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_SNDDATA,
                &mut packets as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(packets, result)
    }
    pub fn get_send_km_state(&self) -> Result<SrtKmState> {
        let mut state = SrtKmState::Unsecured;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_SNDKMSTATE,
                &mut state as *mut SrtKmState as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(state, result)
    }
    pub fn get_send_blocking(&self) -> Result<bool> {
        let mut blocking = true;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_SNDSYN,
                &mut blocking as *mut bool as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(blocking, result)
    }
    pub fn get_send_timeout(&self) -> Result<i32> {
        let mut secs = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_SNDTIMEO,
                &mut secs as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(secs, result)
    }
    pub fn get_socket_state(&self) -> Result<SrtSocketStatus> {
        let mut _optlen = mem::size_of::<srt::SRT_SOCKSTATUS>() as i32;
        let state = unsafe { srt::srt_getsockstate(self.id) };
        let state = match state {
            srt::SRT_SOCKSTATUS::SRTS_INIT => SrtSocketStatus::Init,
            srt::SRT_SOCKSTATUS::SRTS_OPENED => SrtSocketStatus::Opened,
            srt::SRT_SOCKSTATUS::SRTS_LISTENING => SrtSocketStatus::Listening,
            srt::SRT_SOCKSTATUS::SRTS_CONNECTING => SrtSocketStatus::Connecting,
            srt::SRT_SOCKSTATUS::SRTS_CONNECTED => SrtSocketStatus::Connected,
            srt::SRT_SOCKSTATUS::SRTS_BROKEN => SrtSocketStatus::Broken,
            srt::SRT_SOCKSTATUS::SRTS_CLOSING => SrtSocketStatus::Closing,
            srt::SRT_SOCKSTATUS::SRTS_CLOSED => SrtSocketStatus::Closed,
            srt::SRT_SOCKSTATUS::SRTS_NONEXIST => SrtSocketStatus::NonExist,
            _ => return error::handle_result(SrtSocketStatus::Broken, -1),
        };
        error::handle_result(state, 0)
    }
    pub fn get_stream_id(&self) -> Result<String> {
        let mut id = String::from_iter([' '; 512].iter());
        let mut id_len = mem::size_of_val(&id) as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_STATE,
                id.as_mut_ptr() as *mut c_void,
                &mut id_len as *mut c_int,
            )
        };
        id.truncate(id_len as usize);
        error::handle_result(id, result)
    }
    pub fn get_too_late_packet_drop(&self) -> Result<bool> {
        let mut enable = true;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_TLPKTDROP,
                &mut enable as *mut bool as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(enable, result)
    }
    pub fn get_timestamp_based_packet_delivery_mode(&self) -> Result<bool> {
        let mut enable = true;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_TSBPDMODE,
                &mut enable as *mut bool as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(enable, result)
    }
    pub fn get_udp_receive_buffer(&self) -> Result<i32> {
        let mut bytes = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_UDP_RCVBUF,
                &mut bytes as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(bytes, result)
    }
    pub fn get_udp_send_buffer(&self) -> Result<i32> {
        let mut bytes = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_UDP_SNDBUF,
                &mut bytes as *mut c_int as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(bytes, result)
    }
    pub fn get_srt_version(&self) -> Result<i32> {
        let mut version = 0;
        let mut _optlen = mem::size_of::<i32>() as i32;
        let result = unsafe {
            srt::srt_getsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_VERSION,
                &mut version as *mut i32 as *mut c_void,
                &mut _optlen as *mut c_int,
            )
        };
        error::handle_result(version, result)
    }
}
//Post set flag methods
impl SrtSocket {
    pub fn set_time_drift_tracer(&self, enable: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_DRIFTTRACER,
                &enable as *const bool as *const c_void,
                mem::size_of::<i64>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_input_bandwith(&self, bytes_per_sec: i64) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_INPUTBW,
                &bytes_per_sec as *const i64 as *const c_void,
                mem::size_of::<i64>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_recovery_bandwidth_overhead(&self, per_cent: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_OHEADBW,
                &per_cent as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_receive_timeout(&self, msecs: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RCVTIMEO,
                &msecs as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_send_blocking(&self, blocking: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_SNDSYN,
                &blocking as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_send_timeout(&self, msecs: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_SNDTIMEO,
                &msecs as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
}

//Pre set flag methods
impl SrtSocket {
    pub fn set_bind_to_device(&self, device: String) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_BINDTODEVICE,
                device.as_ptr() as *const c_void,
                device.len() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_connection_timeout(&self, msecs: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_CONNTIMEO,
                &msecs as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_flight_flag_size(&self, packets: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_FC,
                &packets as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_ip_type_of_service(&self, type_of_service: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_IPTOS,
                &type_of_service as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_ipv4_time_to_live(&self, hops: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_IPTTL,
                &hops as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_ipv6_only(&self, value: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_IPV6ONLY,
                &value as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_km_refresh_rate(&self, packets: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_KMREFRESHRATE,
                &packets as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_km_preannounce(&self, packets: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_KMPREANNOUNCE,
                &packets as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    #[cfg(target_family = "unix")]
    pub fn set_linger(&self, secs: i32) -> Result<()> {
        let lin = linger {
            l_onoff: (secs > 0) as i32,
            l_linger: secs,
        };
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_LINGER,
                &lin as *const linger as *const c_void,
                mem::size_of::<linger>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    #[cfg(target_os = "windows")]
    pub fn set_linger(&self, secs: i32) -> Result<()> {
        let lin = linger {
            l_onoff: (secs > 0) as u16,
            l_linger: secs as u16,
        };
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_LINGER,
                &lin as *const linger as *const c_void,
                mem::size_of::<linger>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_max_reorder_tolerance(&self, packets: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_LOSSMAXTTL,
                &packets as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_max_bandwith(&self, bytes_per_sec: i64) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_MAXBW,
                &bytes_per_sec as *const i64 as *const c_void,
                mem::size_of::<i64>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_message_api(&self, enable: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_MESSAGEAPI,
                &enable as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_min_version(&self, version: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_MINVERSION,
                &version as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_mss(&self, bytes: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_MSS,
                &bytes as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_nak_report(&self, enable: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_NAKREPORT,
                &enable as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_packet_filter(&self, filter: &str) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_PACKETFILTER,
                filter[..512].as_ptr() as *const c_void,
                filter[..512].len() as i32,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_passphrase(&self, passphrase: &str) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_PASSPHRASE,
                passphrase as *const str as *const c_void,
                passphrase.len() as i32,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_payload_size(&self, bytes: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_PAYLOADSIZE,
                &bytes as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_encryption_key_length(&self, bytes: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_PBKEYLEN,
                &bytes as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_peer_idle_timeout(&self, msecs: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_PEERIDLETIMEO,
                &msecs as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_peer_latency(&self, msecs: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_PEERLATENCY,
                &msecs as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_receive_buffer(&self, bytes: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RCVBUF,
                &bytes as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_receive_latency(&self, msecs: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RCVLATENCY,
                &msecs as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_receive_blocking(&self, blocking: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RCVSYN,
                &blocking as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_rendezvous(&self, rendezvous: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RENDEZVOUS,
                &rendezvous as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_retransmission_algorithm(&self, reduced: bool) -> Result<()> {
        let r = if reduced {
            1
        } else {
            0
        };
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_RETRANSMITALGO,
                &r as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_latency(&self, latency: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_LATENCY,
                &latency as *const i32 as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_sender(&self, sender: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_SENDER,
                &sender as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_reuse_address(&self, reuse: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_REUSEADDR,
                &reuse as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_congestion_controller(&self, controller: SrtCongestionController) -> Result<()> {
        let value = match controller {
            SrtCongestionController::Live => "live",
            SrtCongestionController::File => "file",
        };
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_CONGESTION,
                value.as_ptr() as *const c_void,
                value.len() as i32,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_send_buffer(&self, bytes: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_SNDBUF,
                &bytes as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_send_drop_delay(&self, msecs: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_SNDDROPDELAY,
                &msecs as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_stream_id(&self, id: &str) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_STREAMID,
                id.as_ptr() as *const c_void,
                id.len() as i32,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_enforced_encryption(&self, enforced: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_ENFORCEDENCRYPTION,
                &enforced as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_too_late_packet_drop(&self, enable: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_TLPKTDROP,
                &enable as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_transmission_type(&self, transmission_type: SrtTransmissionType) -> Result<()> {
        let trans_type = match transmission_type {
            SrtTransmissionType::File => srt::SRT_TRANSTYPE::SRTT_FILE,
            SrtTransmissionType::Live => srt::SRT_TRANSTYPE::SRTT_LIVE,
            SrtTransmissionType::Invalid => srt::SRT_TRANSTYPE::SRTT_INVALID,
        };
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_TRANSTYPE,
                &trans_type as *const srt::SRT_TRANSTYPE as *const c_void,
                mem::size_of_val(&trans_type) as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_timestamp_based_packet_delivery_mode(&self, enable: bool) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_TSBPDMODE,
                &enable as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_udp_send_buffer(&self, bytes: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_UDP_SNDBUF,
                &bytes as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
    pub fn set_udp_receive_buffer(&self, bytes: i32) -> Result<()> {
        let result = unsafe {
            srt::srt_setsockflag(
                self.id,
                srt::SRT_SOCKOPT::SRTO_UDP_RCVBUF,
                &bytes as *const c_int as *const c_void,
                mem::size_of::<i32>() as c_int,
            )
        };
        error::handle_result((), result)
    }
}

#[derive(Copy, Clone)]
pub enum SrtKmState {
    Unsecured,
    Securing,
    Secured,
    NoSecret,
    BadSecret,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum SrtTransmissionType {
    Live,
    File,
    Invalid,
}

#[derive(Copy, Clone)]
pub enum SrtCongestionController {
    Live,
    File,
}
