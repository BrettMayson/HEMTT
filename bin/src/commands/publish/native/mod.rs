use std::{
    ffi::CString,
    mem::{MaybeUninit, size_of},
    time::{Duration, Instant},
};

use steamworks_sys as sys;

use crate::Error;

mod item;

const CALL_TIMEOUT: Duration = Duration::from_secs(60);
const UPLOAD_TIMEOUT: Duration = Duration::from_hours(1);
const CALLBACK_INTERVAL: Duration = Duration::from_millis(50);

/// Owns the initialized client and drives its manual callback queue on this thread.
/// The high-level wrapper lacks gallery mutation and Required Item APIs, so all publishing calls use the SDK bindings and one consistent dispatch mechanism.
pub(super) struct SteamWorkshop {
    _client: steamworks::Client,
    ugc: *mut sys::ISteamUGC,
    pipe: sys::HSteamPipe,
}

impl SteamWorkshop {
    pub(super) fn connect() -> Result<Self, Error> {
        sys::load_steam_library()
            .map_err(|error| Error::Workshop(format!("Loading Steam API: {error}")))?;
        let client = super::silence_steam_output(|| steamworks::Client::init_app(super::APP_ID))
            .map_err(|error| {
                Error::Workshop(format!("Initializing Steam (is Steam running?): {error}"))
            })?;
        // SAFETY: the client has initialized the embedded SDK.
        // It stays alive for every native handle and callback through the owning field below.
        let (ugc, pipe) = unsafe { (sys::SteamAPI_SteamUGC_v021(), sys::SteamAPI_GetHSteamPipe()) };
        if ugc.is_null() || pipe == 0 {
            return Err(Error::Workshop(
                "Steam returned an invalid UGC interface or client pipe".to_string(),
            ));
        }
        Ok(Self {
            _client: client,
            ugc,
            pipe,
        })
    }

    /// Wait for one typed call result, using the manual dispatch mode initialized by `steamworks::Client`.
    /// No background dispatcher may consume this queue.
    fn wait<T: Copy>(
        &self,
        call: sys::SteamAPICall_t,
        callback_id: i32,
        timeout: Duration,
        operation: &str,
    ) -> Result<T, Error> {
        if call == 0 {
            return Err(Error::Workshop(format!(
                "{operation}: Steam returned an invalid API call"
            )));
        }
        let started = Instant::now();
        loop {
            // SAFETY: this object's client owns the live pipe.
            // Callback storage is borrowed only until FreeLastCallback, called exactly once below.
            unsafe {
                sys::SteamAPI_ManualDispatch_RunFrame(self.pipe);
                let mut message: sys::CallbackMsg_t = std::mem::zeroed();
                while sys::SteamAPI_ManualDispatch_GetNextCallback(self.pipe, &raw mut message) {
                    let result = if message.m_iCallback
                        == sys::SteamAPICallCompleted_t_k_iCallback as i32
                        && usize::try_from(message.m_cubParam).ok()
                            == Some(size_of::<sys::SteamAPICallCompleted_t>())
                        && !message.m_pubParam.is_null()
                    {
                        let completed = message
                            .m_pubParam
                            .cast::<sys::SteamAPICallCompleted_t>()
                            .read_unaligned();
                        if completed.m_hAsyncCall == call {
                            Some(self.call_result::<T>(&completed, callback_id, operation))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    sys::SteamAPI_ManualDispatch_FreeLastCallback(self.pipe);
                    if let Some(result) = result {
                        return result;
                    }
                }
            }
            if started.elapsed() >= timeout {
                return Err(Error::Workshop(format!(
                    "{operation}: no Steam result received within {} seconds",
                    timeout.as_secs()
                )));
            }
            std::thread::sleep(CALLBACK_INTERVAL);
        }
    }

    fn call_result<T: Copy>(
        &self,
        completed: &sys::SteamAPICallCompleted_t,
        callback_id: i32,
        operation: &str,
    ) -> Result<T, Error> {
        if completed.m_iCallback != callback_id || completed.m_cubParam as usize != size_of::<T>() {
            return Err(Error::Workshop(format!(
                "{operation}: unexpected Steam callback type or size"
            )));
        }
        let size =
            i32::try_from(size_of::<T>()).map_err(|error| Error::Workshop(error.to_string()))?;
        let mut result = MaybeUninit::<T>::uninit();
        let mut failed = false;
        // SAFETY: the callback ID and exact struct size were checked, and Steam writes the result into properly aligned storage before assume_init.
        let received = unsafe {
            sys::SteamAPI_ManualDispatch_GetAPICallResult(
                self.pipe,
                completed.m_hAsyncCall,
                result.as_mut_ptr().cast(),
                size,
                callback_id,
                &raw mut failed,
            )
        };
        if !received || failed {
            return Err(Error::Workshop(format!(
                "{operation}: failed to retrieve Steam API result"
            )));
        }
        Ok(unsafe { result.assume_init() })
    }
}

fn checked(ok: bool, operation: &str) -> Result<(), Error> {
    if ok {
        Ok(())
    } else {
        Err(Error::Workshop(format!("Steam rejected {operation}")))
    }
}

fn succeeded(result: sys::EResult, operation: &str) -> Result<(), Error> {
    if result == sys::EResult::k_EResultOK {
        Ok(())
    } else {
        Err(Error::Workshop(format!(
            "{operation}: {}",
            steamworks::SteamError::from(result)
        )))
    }
}

fn string(value: &str) -> Result<CString, Error> {
    CString::new(value).map_err(|error| Error::Workshop(format!("Invalid Steam string: {error}")))
}

fn path_string(path: &std::path::Path) -> Result<CString, Error> {
    let path = path
        .to_str()
        .ok_or_else(|| Error::Workshop("Steam paths must be UTF-8".to_string()))?;
    string(path)
}
