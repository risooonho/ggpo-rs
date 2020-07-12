use crate::game_input::Frame;
use crate::player::{Player, PlayerHandle};
use bytes::{Bytes, BytesMut};
use log::info;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GGPOError {
    #[error("GGPO OK.")]
    Ok,
    #[error("GGPO Success.")]
    Success,
    #[error("GGPO general Failure.")]
    GeneralFailure,
    #[error("GGPO invalid session.")]
    InvalidSession,
    #[error("GGPO invalid player handle.")]
    InvalidPlayerHandle,
    #[error("GGPO player out of range.")]
    PlayerOutOfRange,
    #[error("GGPO prediction threshold.")]
    PredictionThreshold,
    #[error("GGPO unsupported.")]
    Unsupported,
    #[error("GGPO not synchronized.")]
    NotSynchronized,
    #[error("GGPO in rollback.")]
    InRollback,
    #[error("GGPO input dropped.")]
    InputDropped,
    #[error("GGPO player disconnected.")]
    PlayerDisconnected,
    #[error("GGPO too many spectators.")]
    TooManySpectators,
    #[error("GGPO invalid request.")]
    InvalidRequest,
}

pub enum Event {
    ConnectedToPeer {
        player: PlayerHandle,
    },
    SynchronizingWithPeer {
        count: i32,
        total: i32,
    },
    SynchronizedWithPeer {
        player: PlayerHandle,
    },
    Running {},
    DisconnectedFromPeer {
        player: PlayerHandle,
    },
    Timesync {
        frames_ahead: i32,
    },
    ConnectionInterrupted {
        player: PlayerHandle,
        disconnect_timeout: i32,
    },
    ConnectionResumed {
        player: PlayerHandle,
    },
}

pub trait Session {
    fn do_poll(_timeout: usize) -> Result<(), GGPOError> {
        Ok(())
    }

    fn add_player(player: Player, handle: PlayerHandle) -> Result<(), GGPOError> {
        Ok(())
    }

    fn add_local_input(player: PlayerHandle, values: String, size: usize) -> Result<(), GGPOError> {
        Ok(())
    }

    fn synchronize_input(
        values: String,
        size: usize,
        disconnect_flags: i32,
    ) -> Result<(), GGPOError> {
        Ok(())
    }

    fn increment_frame() -> Result<(), GGPOError> {
        Ok(())
    }

    fn chat(_text: String) -> Result<(), GGPOError> {
        Ok(())
    }

    fn disconnect_player(_handle: PlayerHandle) -> Result<(), GGPOError> {
        Ok(())
    }

    fn get_network_stats(_stats: NetworkStats, _handle: PlayerHandle) -> Result<(), GGPOError> {
        Ok(())
    }

    //TODO: stub this with the log crate
    fn logv(fmt: &str) -> Result<(), GGPOError> {
        Ok(())
    }

    fn set_frame_delay(_player: PlayerHandle, _delay: i32) -> Result<(), GGPOError> {
        Err(GGPOError::Unsupported)
    }

    fn set_disconnect_timeout(_timeout: usize) -> Result<(), GGPOError> {
        Err(GGPOError::Unsupported)
    }

    fn set_disconnect_notify_start(_timeout: usize) -> Result<(), GGPOError> {
        Err(GGPOError::Unsupported)
    }
}

pub trait GGPOSessionCallbacks: Clone + Sized {
    // was deprecated anyway
    // fn begin_game() -> bool;

    /*
     * save_game_state - The client should allocate a buffer, copy the
     * entire contents of the current game state into it, and copy the
     * length into the *len parameter.  Optionally, the client can compute
     * a checksum of the data and store it in the *checksum argument.
     */
    fn save_game_state(
        &mut self,
        buffer: &Bytes,
        length: &usize,
        checksum: Option<u32>,
        frame: Frame,
    ) -> bool;

    /*
     * load_game_state - GGPO.net will call this function at the beginning
     * of a rollback.  The buffer and len parameters contain a previously
     * saved state returned from the save_game_state function.  The client
     * should make the current game state match the state contained in the
     * buffer.
     */
    fn load_game_state(&mut self, buffer: &Bytes, length: usize) -> bool;

    /*
     * log_game_state - Used in diagnostic testing.  The client should use
     * the ggpo_log function to write the contents of the specified save
     * state in a human readible form.
     */
    fn log_game_state(&mut self, filename: String, buffer: Bytes, length: usize) -> bool;

    /*
     * free_buffer - Frees a game state allocated in save_game_state.  You
     * should deallocate the memory contained in the buffer.
     */
    fn free_buffer(&mut self, buffer: &Bytes);

    /*
     * advance_frame - Called during a rollback.  You should advance your game
     * state by exactly one frame.  Before each frame, call ggpo_synchronize_input
     * to retrieve the inputs you should use for that frame.  After each frame,
     * you should call ggpo_advance_frame to notify GGPO.net that you're
     * finished.
     *
     * The flags parameter is reserved.  It can safely be ignored at this time.
     */
    fn advance_frame(&mut self, flags: i32) -> bool;

    /*
     * on_event - Notification that something has happened.  See the GGPOEventCode
     * structure above for more information.
     */
    fn on_event(&mut self, info: &Event);
}
#[no_mangle]
pub struct CallbacksStub {
    /*
     * save_game_state - The client should allocate a buffer, copy the
     * entire contents of the current game state into it, and copy the
     * length into the *len parameter.  Optionally, the client can compute
     * a checksum of the data and store it in the *checksum argument.
     */
    pub save_game_state: extern "C" fn(
        buffer: Option<BytesMut>,
        length: usize,
        checksum: Option<u32>,
        frame: Frame,
    ) -> bool,

    /*
     * load_game_state - GGPO.net will call this function at the beginning
     * of a rollback.  The buffer and len parameters contain a previously
     * saved state returned from the save_game_state function.  The client
     * should make the current game state match the state contained in the
     * buffer.
     */
    pub load_game_state: extern "C" fn(buffer: BytesMut, length: usize) -> bool,

    /*
     * log_game_state - Used in diagnostic testing.  The client should use
     * the ggpo_log function to write the contents of the specified save
     * state in a human readible form.
     */
    pub log_game_state: extern "C" fn(filename: String, buffer: BytesMut, length: usize) -> bool,

    /*
     * free_buffer - Frees a game state allocated in save_game_state.  You
     * should deallocate the memory contained in the buffer.
     */
    pub free_buffer: extern "C" fn(buffer: BytesMut),

    /*
     * advance_frame - Called during a rollback.  You should advance your game
     * state by exactly one frame.  Before each frame, call ggpo_synchronize_input
     * to retrieve the inputs you should use for that frame.  After each frame,
     * you should call ggpo_advance_frame to notify GGPO.net that you're
     * finished.
     *
     * The flags parameter is reserved.  It can safely be ignored at this time.
     */
    pub advance_frame: extern "C" fn(flags: i32) -> bool,

    /*
     * on_event - Notification that something has happened.  See the GGPOEventCode
     * structure above for more information.
     */
    pub on_event: extern "C" fn(info: &Event),
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Network {
    pub send_queue_len: usize,
    pub recv_queue_len: usize,
    pub ping: usize,
    pub kbps_sent: usize,
}

impl Network {
    pub const fn new() -> Self {
        Self {
            send_queue_len: 0,
            recv_queue_len: 0,
            ping: 0,
            kbps_sent: 0,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TimeSync {
    pub local_frames_behind: i32,
    pub remote_frames_behind: i32,
}

impl TimeSync {
    pub const fn new() -> Self {
        Self {
            local_frames_behind: 0,
            remote_frames_behind: 0,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct NetworkStats {
    pub network: Network,
    pub timesync: TimeSync,
}

impl NetworkStats {
    pub const fn new() -> Self {
        Self {
            network: Network::new(),
            timesync: TimeSync::new(),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct LocalEndpoint {
    player_num: usize,
}
