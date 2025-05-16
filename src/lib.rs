#![no_std]

mod serial_buffer;
pub use serial_buffer::*;

#[derive(Copy, Clone, Debug)]
pub enum Parameter {
    DelayCompensation,
    StartupFrequency,
    RunMode,
    LockTime,
    StartupTime,
    OnTime,
    RampStart,
    RampEnd,
    MinLockCurrent,
    CurrentLimit,
}

impl Parameter {
    fn sign_extend(&self) -> bool {
        match self {
            Self::DelayCompensation => true,
            Self::StartupFrequency  |
            Self::RunMode           |
            Self::LockTime          |
            Self::StartupTime       |
            Self::OnTime            |
            Self::RampStart         |
            Self::RampEnd           |
            Self::MinLockCurrent    |
            Self::CurrentLimit      => false,
        }
    }
}

const PARAMETER_ID_DELAY_COMP       : u8 = 1;
const PARAMETER_ID_STARTUP_FREQ     : u8 = 2;
const PARAMETER_ID_RUN_MODE         : u8 = 3;
const PARAMETER_ID_LOCK_TIME        : u8 = 4;
const PARAMETER_ID_STARTUP_TIME     : u8 = 5;
const PARAMETER_ID_ON_TIME          : u8 = 6;
const PARAMETER_ID_RAMP_START       : u8 = 7;
const PARAMETER_ID_RAMP_END         : u8 = 8;
const PARAMETER_ID_MIN_LOCK_CURRENT : u8 = 9;
const PARAMETER_ID_CURRENT_LIMIT    : u8 = 10;

impl Into<u8> for Parameter {
    fn into(self) -> u8 {
        match self {
            Self::DelayCompensation   => PARAMETER_ID_DELAY_COMP,
            Self::StartupFrequency    => PARAMETER_ID_STARTUP_FREQ,
            Self::RunMode             => PARAMETER_ID_RUN_MODE,
            Self::LockTime            => PARAMETER_ID_LOCK_TIME,
            Self::StartupTime         => PARAMETER_ID_STARTUP_TIME,
            Self::OnTime              => PARAMETER_ID_ON_TIME,
            Self::RampStart           => PARAMETER_ID_RAMP_START,
            Self::RampEnd             => PARAMETER_ID_RAMP_END,
            Self::MinLockCurrent      => PARAMETER_ID_MIN_LOCK_CURRENT,
            Self::CurrentLimit        => PARAMETER_ID_CURRENT_LIMIT,
        }
    }
}

impl TryFrom<u8> for Parameter {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        Ok(match value {
            PARAMETER_ID_DELAY_COMP       => Self::DelayCompensation,
            PARAMETER_ID_STARTUP_FREQ     => Self::StartupFrequency,
            PARAMETER_ID_RUN_MODE         => Self::RunMode,
            PARAMETER_ID_LOCK_TIME        => Self::LockTime,
            PARAMETER_ID_STARTUP_TIME     => Self::StartupTime,
            PARAMETER_ID_ON_TIME          => Self::OnTime,
            PARAMETER_ID_RAMP_START       => Self::RampStart,
            PARAMETER_ID_RAMP_END         => Self::RampEnd,
            PARAMETER_ID_MIN_LOCK_CURRENT => Self::MinLockCurrent,
            PARAMETER_ID_CURRENT_LIMIT    => Self::CurrentLimit,
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

const MESSAGE_START_BIT: u8 = 0x80;

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
                    buffer.push(CONTROLLER_MESSAGE_ID_SET_DEBUG_LED | MESSAGE_START_BIT);
                    buffer.push(if *state { 1 } else { 0 });
                }
                Self::GetParam(param) => {
                    buffer.push(CONTROLLER_MESSAGE_ID_GET_PARAM | MESSAGE_START_BIT);
                    buffer.push((*param).into());
                },
                Self::SetParam(param, value) => {
                    buffer.push(CONTROLLER_MESSAGE_ID_SET_PARAM | MESSAGE_START_BIT);
                    buffer.push((*param).into());
                    buffer.push(((value >> 0) & 0x7F) as u8);
                    buffer.push(((value >> 7) & 0x7F) as u8);
                },
                Self::GetStat(stat) => {
                    buffer.push(CONTROLLER_MESSAGE_ID_GET_STAT | MESSAGE_START_BIT);
                    buffer.push((*stat).into());
                },
                Self::ResetStats => {
                    buffer.push(CONTROLLER_MESSAGE_ID_RESET_STATS | MESSAGE_START_BIT);
                }
                Self::Ping(seq) => {
                    buffer.push(CONTROLLER_MESSAGE_ID_PING | MESSAGE_START_BIT);
                    buffer.push(((*seq >>  0) & 0x7F) as u8);
                    buffer.push(((*seq >>  7) & 0x7F) as u8);
                    buffer.push(((*seq >> 14) & 0x7F) as u8);
                    buffer.push(((*seq >> 21) & 0x7F) as u8);
                },
            }
            true
        } else {
            false
        }
    }
    
    pub fn try_receive<const N: usize>(rx_buffer: &mut SerialBuffer<N>) -> Result<Option<Self>, ()> {
        while let Some(id_byte) = rx_buffer.peek() {
            if (id_byte & MESSAGE_START_BIT) != 0 {
                break;
            }
            rx_buffer.pop();
        }
        if let Some(id) = rx_buffer.peek() {
            let id = id & !MESSAGE_START_BIT;
            let length = match id {
                CONTROLLER_MESSAGE_ID_SET_DEBUG_LED => 2,
                CONTROLLER_MESSAGE_ID_GET_PARAM => 2,
                CONTROLLER_MESSAGE_ID_SET_PARAM => 4,
                CONTROLLER_MESSAGE_ID_GET_STAT => 2,
                CONTROLLER_MESSAGE_ID_RESET_STATS => 1,
                CONTROLLER_MESSAGE_ID_PING => 5,
                _ => {
                    rx_buffer.pop();
                    Err(())?
                }
            };
            if rx_buffer.count() >= length {
                rx_buffer.pop();
                match id {
                    CONTROLLER_MESSAGE_ID_SET_DEBUG_LED => {
                        let state = rx_buffer.pop().unwrap();
                        return Ok(Some(ControllerMessage::SetDebugLed(state != 0)));
                    },
                    CONTROLLER_MESSAGE_ID_GET_PARAM => {
                        let param_id = rx_buffer.pop().unwrap();
                        return Ok(Some(ControllerMessage::GetParam(Parameter::try_from(param_id)?)));
                    },
                    CONTROLLER_MESSAGE_ID_SET_PARAM => {
                        let param_id = rx_buffer.pop().unwrap();
                        let mut param_value = 
                            ((rx_buffer.pop().unwrap() as u16) << 0) |
                            ((rx_buffer.pop().unwrap() as u16) << 7);
                        let param = Parameter::try_from(param_id)?;
                        if param.sign_extend() && (param_value & 0x20) != 0 {
                            param_value |= 0xC000;
                        }
                        return Ok(Some(ControllerMessage::SetParam(param, param_value)));
                    },
                    CONTROLLER_MESSAGE_ID_GET_STAT => {
                        let param_id = rx_buffer.pop().unwrap();
                        return Ok(Some(ControllerMessage::GetStat(Statistic::try_from(param_id)?)));
                    },
                    CONTROLLER_MESSAGE_ID_RESET_STATS => {
                        return Ok(Some(ControllerMessage::ResetStats));
                    },
                    CONTROLLER_MESSAGE_ID_PING => {
                        let seq = 
                            (rx_buffer.pop().unwrap() as u32) << 0  |
                            (rx_buffer.pop().unwrap() as u32) << 7  |
                            (rx_buffer.pop().unwrap() as u32) << 14 |
                            (rx_buffer.pop().unwrap() as u32) << 21;
                        return Ok(Some(ControllerMessage::Ping(seq)));
                    },
                    _ => unreachable!()
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
                    tx_buffer.push(REMOTE_MESSAGE_ID_PING | MESSAGE_START_BIT);
                    tx_buffer.push(((seq >>  0) & 0x7F) as u8);
                    tx_buffer.push(((seq >>  7) & 0x7F) as u8);
                    tx_buffer.push(((seq >> 14) & 0x7F) as u8);
                    tx_buffer.push(((seq >> 21) & 0x7F) as u8);
                    true
                } else {
                    false
                }
            },

            Self::GetParamResult(param, value) => {
                if tx_buffer.free_space() >= 4 {
                    tx_buffer.push(REMOTE_MESSAGE_ID_GET_PARAM_RESULT | MESSAGE_START_BIT);
                    tx_buffer.push((*param).into());
                    tx_buffer.push(((*value >>  0) & 0x7F) as u8);
                    tx_buffer.push(((*value >>  7) & 0x7F) as u8);
                    true
                } else {
                    false
                }
            },
            Self::GetStatResult(stat, value) => {
                if tx_buffer.free_space() >= 4 {
                    tx_buffer.push(REMOTE_MESSAGE_ID_GET_STAT_RESULT | MESSAGE_START_BIT);
                    tx_buffer.push((*stat).into());
                    tx_buffer.push(((*value >>  0) & 0xFF) as u8);
                    tx_buffer.push(((*value >>  8) & 0xFF) as u8);
                    true
                } else {
                    false
                }
            },
        }
    }

    pub fn try_receive<const N: usize>(rx_buffer: &mut SerialBuffer<N>) -> Result<Option<Self>, ()> {
        while let Some(id_byte) = rx_buffer.peek() {
            if (id_byte & MESSAGE_START_BIT) != 0 {
                break;
            }
            rx_buffer.pop();
        }
        if let Some(id) = rx_buffer.peek() {
            let id = id & !MESSAGE_START_BIT;
            let length = match id  {
                REMOTE_MESSAGE_ID_GET_PARAM_RESULT => 4,
                REMOTE_MESSAGE_ID_GET_STAT_RESULT  => 4,
                REMOTE_MESSAGE_ID_PING             => 5,
                _ => { rx_buffer.pop(); return Err(()) }
            };
            if rx_buffer.count() >= length {
                _ = rx_buffer.pop();
                match id {
                    REMOTE_MESSAGE_ID_GET_PARAM_RESULT => {
                        let param_id = rx_buffer.pop().unwrap();
                        let mut param_value = 
                            ((rx_buffer.pop().unwrap() as u16) << 0) |
                            ((rx_buffer.pop().unwrap() as u16) << 7);
                        let param = Parameter::try_from(param_id)?;
                        if param.sign_extend() && (param_value & 0x20) != 0 {
                            param_value |= 0xC000;
                        }
                        return Ok(Some(RemoteMessage::GetParamResult(param, param_value)));
                    },
                    REMOTE_MESSAGE_ID_GET_STAT_RESULT => {
                        let stat = Statistic::try_from(rx_buffer.pop().unwrap())?;
                        let value =
                            ((rx_buffer.pop().unwrap() as u16) <<  0) |
                            ((rx_buffer.pop().unwrap() as u16) <<  7);
                        Ok(Some(Self::GetStatResult(stat, value)))
                    },
                    REMOTE_MESSAGE_ID_PING => {
                        let seq = 
                            ((rx_buffer.pop().unwrap() as u32) <<  0) |
                            ((rx_buffer.pop().unwrap() as u32) <<  7) |
                            ((rx_buffer.pop().unwrap() as u32) << 14) |
                            ((rx_buffer.pop().unwrap() as u32) << 21);
                        Ok(Some(Self::Ping(seq)))
                    },
                    _ => unreachable!()
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
