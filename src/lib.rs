mod serial_buffer;
pub use serial_buffer::*;

#[derive(Copy, Clone, Debug)]
pub enum Parameter {
    LedState,
    DelayCompensationNS,
    StartupFrequency,
}

const PARAMETER_ID_LED_STATE: u8 = 0;
const PARAMETER_ID_DELAY_COMP_NS: u8 = 1;
const PARAMETER_ID_STARTUP_FREQ: u8 = 2;

impl Into<u8> for Parameter {
    fn into(self) -> u8 {
        match self {
            Self::LedState            => PARAMETER_ID_LED_STATE,
            Self::DelayCompensationNS => PARAMETER_ID_DELAY_COMP_NS,
            Self::StartupFrequency    => PARAMETER_ID_STARTUP_FREQ,
        }
    }
}

impl TryFrom<u8> for Parameter {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        Ok(match value {
            PARAMETER_ID_LED_STATE     => Self::LedState,
            PARAMETER_ID_DELAY_COMP_NS => Self::DelayCompensationNS,
            PARAMETER_ID_STARTUP_FREQ  => Self::StartupFrequency,
            _ => return Err(())
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Statistic {
    MaxPrimaryCurrent,
}

const STATISTIC_ID_MAX_PRIMARY_CURRENT: u8 = 0;

impl Into<u8> for Statistic {
    fn into(self) -> u8 {
        match self {
            Self::MaxPrimaryCurrent => STATISTIC_ID_MAX_PRIMARY_CURRENT,
        }
    }
}

impl TryFrom<u8> for Statistic {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        Ok(match value {
            STATISTIC_ID_MAX_PRIMARY_CURRENT => Self::MaxPrimaryCurrent,
            _ => return Err(())
        })
    }
}


#[derive(Copy, Clone, Debug)]
pub enum ControllerMessage {
    SetDebugLed(bool),
    GetParam(Parameter),
    SetParam(Parameter, u16),
    GetStat(Statistic),
    ResetStats,
    Ping(u32),
}

const CONTROLLER_MESSAGE_ID_SET_DEBUG_LED: u8 = 0;
const CONTROLLER_MESSAGE_ID_GET_PARAM: u8 = 1;
const CONTROLLER_MESSAGE_ID_SET_PARAM: u8 = 2;
const CONTROLLER_MESSAGE_ID_GET_STAT: u8 = 3;
const CONTROLLER_MESSAGE_ID_RESET_STATS: u8 = 4;
const CONTROLLER_MESSAGE_ID_PING: u8 = 0xFF;

impl ControllerMessage {
    pub fn try_send<const N: usize>(&self, buffer: &mut SerialBuffer<N>) -> bool {
        let length = match self {
            Self::SetDebugLed(..) => 2,
            Self::GetParam(..)    => 2,
            Self::SetParam(..)    => 4,
            Self::GetStat(..)     => 2,
            Self::ResetStats      => 1,
            Self::Ping(..)        => 5,
        };
        if buffer.free_space() >= length {
            match self {
                Self::SetDebugLed(state) => {
                    buffer.push(CONTROLLER_MESSAGE_ID_SET_DEBUG_LED);
                    buffer.push(if *state { 1 } else { 0 });
                }
                Self::GetParam(param) => {
                    buffer.push(CONTROLLER_MESSAGE_ID_GET_PARAM);
                    buffer.push((*param).into());
                },
                Self::SetParam(param, value) => {
                    buffer.push(CONTROLLER_MESSAGE_ID_SET_PARAM);
                    buffer.push((*param).into());
                    buffer.push(((value >> 0) & 0xFF) as u8);
                    buffer.push(((value >> 8) & 0xFF) as u8);
                },
                Self::GetStat(stat) => {
                    buffer.push(CONTROLLER_MESSAGE_ID_GET_STAT);
                    buffer.push((*stat).into());
                },
                Self::ResetStats => {
                    buffer.push(CONTROLLER_MESSAGE_ID_RESET_STATS);
                }
                Self::Ping(seq) => {
                    buffer.push(CONTROLLER_MESSAGE_ID_PING);
                    buffer.push(((*seq >>  0) & 0xFF) as u8);
                    buffer.push(((*seq >>  8) & 0xFF) as u8);
                    buffer.push(((*seq >> 16) & 0xFF) as u8);
                    buffer.push(((*seq >> 24) & 0xFF) as u8);
                },
            }
            true
        } else {
            false
        }
    }
    
    pub fn try_receive<const N: usize>(rx_buffer: &mut SerialBuffer<N>) -> Result<Option<Self>, ()> {
        if let Some(id) = rx_buffer.peek() {
            match id {
                CONTROLLER_MESSAGE_ID_SET_DEBUG_LED => {
                    if rx_buffer.count() >= 2 {
                        rx_buffer.pop();
                        let state = rx_buffer.pop().unwrap();
                        return Ok(Some(ControllerMessage::SetDebugLed(state != 0)))
                    } 
                },
                CONTROLLER_MESSAGE_ID_PING => {
                    if rx_buffer.count() >= 5 {
                        rx_buffer.pop();
                        let seq = 
                            (rx_buffer.pop().ok_or(())? as u32) << 0  |
                            (rx_buffer.pop().ok_or(())? as u32) << 8  |
                            (rx_buffer.pop().ok_or(())? as u32) << 16 |
                            (rx_buffer.pop().ok_or(())? as u32) << 24;
                        return Ok(Some(ControllerMessage::Ping(seq)))
                    }
                },
                CONTROLLER_MESSAGE_ID_GET_PARAM => {
                    todo!()
                },
                CONTROLLER_MESSAGE_ID_SET_PARAM => {
                    todo!()
                },
                CONTROLLER_MESSAGE_ID_GET_STAT => {
                    todo!()
                },
                CONTROLLER_MESSAGE_ID_RESET_STATS => {
                    todo!()
                },
                _ => {
                    rx_buffer.pop();
                    return Err(());
                }
            }
        }
        Ok(None)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RemoteMessage {
    GetParamResult(Parameter, u16),
    GetStatResult(Statistic, u16),
    Ping(u32),
}

const REMOTE_MESSAGE_ID_GET_PARAM_RESULT: u8 = 0;
const REMOTE_MESSAGE_ID_GET_STAT_RESULT: u8 = 1;
const REMOTE_MESSAGE_ID_PING: u8 = 0xFF;

impl RemoteMessage {
    pub fn try_send<const N: usize>(&self, tx_buffer: &mut SerialBuffer<N>) -> bool {
        match self {
            Self::Ping(seq) => {
                if tx_buffer.free_space() >= 5 {
                    tx_buffer.push(REMOTE_MESSAGE_ID_PING);
                    tx_buffer.push(((seq >>  0) & 0xFF) as u8);
                    tx_buffer.push(((seq >>  8) & 0xFF) as u8);
                    tx_buffer.push(((seq >> 16) & 0xFF) as u8);
                    tx_buffer.push(((seq >> 24) & 0xFF) as u8);
                    true
                } else {
                    false
                }
            },

            Self::GetParamResult(param, value) => {
                if tx_buffer.free_space() >= 4 {
                    todo!()
                } else {
                    false
                }
            },
            Self::GetStatResult(stat, value) => {
                if tx_buffer.free_space() >= 4 {
                    todo!()
                } else {
                    false
                }
            },
        }
    }

    pub fn try_receive<const N: usize>(buffer: &mut SerialBuffer<N>) -> Option<Self> {
        if let Some(id) = buffer.peek() {
            let length = match id {
                REMOTE_MESSAGE_ID_GET_PARAM_RESULT => 4,
                REMOTE_MESSAGE_ID_GET_STAT_RESULT  => 4,
                REMOTE_MESSAGE_ID_PING             => 5,
                _ => { buffer.pop(); return None }
            };
            if buffer.count() >= length {
                match buffer.pop().unwrap() {
                    REMOTE_MESSAGE_ID_GET_PARAM_RESULT => todo!(),
                    REMOTE_MESSAGE_ID_GET_STAT_RESULT => todo!(),
                    REMOTE_MESSAGE_ID_PING => {
                        let seq = 
                            ((buffer.pop().unwrap() as u32) <<  0) |
                            ((buffer.pop().unwrap() as u32) <<  8) |
                            ((buffer.pop().unwrap() as u32) << 16) |
                            ((buffer.pop().unwrap() as u32) << 24);
                        Some(Self::Ping(seq))
                    },
                    _ => None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}
