#![no_std]

mod serial_buffer;
pub use serial_buffer::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Parameter {
    DelayCompensation,
    StartupFrequency,
    LockRange,
    RunMode,
    LockTime,
    StartupTime,
    OnTime,
    OffTime,
    RampStartPower,
    RampEndPower,
    MinLockCurrent,
    CurrentLimit,
    FlatPower,
}

#[derive(Copy, Clone, Debug)]
pub enum ParameterValue {
    DelayCompensationNS(i16),
    StartupFrequencykHz(f32),
    LockRangekHz(f32),
    RunMode(RunMode),
    LockTimeUs(u16),
    StartupTimeUs(u16),
    OnTimeUs(u16),
    OffTimeMs(u16),
    RampStartPower(f32),
    RampEndPower(f32),
    MinLockCurrentA(f32),
    CurrentLimitA(f32),
    FlatPower(f32),
}

impl ParameterValue {
    pub fn parameter(&self) -> Parameter {
        match self {
            Self::DelayCompensationNS(..) => Parameter::CurrentLimit,
            Self::StartupFrequencykHz(..) => Parameter::StartupFrequency,
            Self::LockRangekHz(..) => Parameter::LockRange,
            Self::RunMode(..) => Parameter::RunMode,
            Self::LockTimeUs(..) => Parameter::LockTime,
            Self::StartupTimeUs(..) => Parameter::StartupTime,
            Self::OnTimeUs(..) => Parameter::OnTime,
            Self::OffTimeMs(..) => Parameter::OffTime,
            Self::RampStartPower(..) => Parameter::RampStartPower,
            Self::RampEndPower(..) => Parameter::RampEndPower,
            Self::MinLockCurrentA(..) => Parameter::MinLockCurrent,
            Self::CurrentLimitA(..) => Parameter::CurrentLimit,
            Self::FlatPower(..) => Parameter::FlatPower,
        }
    }
}

fn sign_extend_i14(x: u16) -> i16 {
    if (x & 0x2000) != 0 {
        (x | 0xC000) as i16
    } else {
        x as i16
    }
}

impl TryFrom<(Parameter, u16)> for ParameterValue {
    type Error = ();
    fn try_from(x: (Parameter, u16)) -> Result<Self, ()> {
        let (param, value) = x;
        Ok(match param {
            Parameter::DelayCompensation => Self::DelayCompensationNS(sign_extend_i14(value)),
            Parameter::StartupFrequency => Self::StartupFrequencykHz(value as f32 / 16.0),
            Parameter::LockRange => Self::LockRangekHz(value as f32 / 16.0),
            Parameter::RunMode => Self::RunMode(RunMode::try_from(value)?),
            Parameter::LockTime => Self::LockTimeUs(value),
            Parameter::StartupTime => Self::StartupTimeUs(value),
            Parameter::OnTime => Self::OnTimeUs(value * 10),
            Parameter::OffTime => Self::OffTimeMs(value),
            Parameter::RampStartPower => Self::RampStartPower(value as f32 / 16383.0),
            Parameter::RampEndPower => Self::RampEndPower(value as f32 / 16383.0),
            Parameter::MinLockCurrent => Self::MinLockCurrentA(value as f32 / 256.0),
            Parameter::CurrentLimit => Self::CurrentLimitA(value as f32 / 32.0),
            Parameter::FlatPower => Self::FlatPower(value as f32 / 16383.0),
        })
    }
}

impl Into<(Parameter, u16)> for ParameterValue {
    fn into(self) -> (Parameter, u16) {
        match self {
            Self::DelayCompensationNS(delay_ns)      => (Parameter::DelayCompensation, delay_ns as u16                                   ),
            Self::StartupFrequencykHz(frequency_khz) => (Parameter::StartupFrequency,  (frequency_khz * 16.0) as u16                     ),
            Self::LockRangekHz(frequency_khz)        => (Parameter::LockRange,         (frequency_khz * 16.0) as u16                     ),
            Self::RunMode(run_mode)                  => (Parameter::RunMode,           run_mode.into()                                   ),
            Self::LockTimeUs(time)                   => (Parameter::LockTime,          time                                              ),
            Self::StartupTimeUs(time)                => (Parameter::StartupTime,       time                                              ),
            Self::OnTimeUs(time)                     => (Parameter::OnTime,            time / 10                                         ),
            Self::OffTimeMs(time)                    => (Parameter::OffTime,           time                                              ),
            Self::RampStartPower(power)              => (Parameter::RampStartPower,    ((power * 16384.0) as i32).clamp(0, 0x3FFF) as u16),
            Self::RampEndPower(power)                => (Parameter::RampEndPower,      ((power * 16384.0) as i32).clamp(0, 0x3FFF) as u16),
            Self::MinLockCurrentA(current)           => (Parameter::MinLockCurrent,    ((current * 256.0) as i32).clamp(0, 0x3FFF) as u16),
            Self::CurrentLimitA(current)             => (Parameter::CurrentLimit,      ((current * 32.0) as i32).clamp(0, 0x3FFF) as u16 ),
            Self::FlatPower(power)                   => (Parameter::FlatPower,         ((power * 16384.0) as i32).clamp(0, 0x3FFF) as u16),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RunMode {
    OpenLoop,
    TestClosedLoop,
    ClosedLoopRamp,
}

impl Into<u16> for RunMode {
    fn into(self) -> u16 {
        match self {
            Self::OpenLoop        => 0,
            Self::TestClosedLoop  => 1,
            Self::ClosedLoopRamp  => 2,
        }
    }
}

impl TryFrom<u16> for RunMode {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, ()> {
        Ok(match value {
            0 => Self::OpenLoop,
            1 => Self::TestClosedLoop,
            2 => Self::ClosedLoopRamp,
            _ => return Err(()),
        })
    }
}

const PARAMETER_ID_DELAY_COMP       : u8 = 1;
const PARAMETER_ID_STARTUP_FREQ     : u8 = 2;
const PARAMETER_ID_RUN_MODE         : u8 = 3;
const PARAMETER_ID_LOCK_TIME        : u8 = 4;
const PARAMETER_ID_STARTUP_TIME     : u8 = 5;
const PARAMETER_ID_ON_TIME          : u8 = 6;
const PARAMETER_ID_OFF_TIME         : u8 = 7;
const PARAMETER_ID_RAMP_START       : u8 = 8;
const PARAMETER_ID_RAMP_END         : u8 = 9;
const PARAMETER_ID_MIN_LOCK_CURRENT : u8 = 10;
const PARAMETER_ID_CURRENT_LIMIT    : u8 = 11;
const PARAMETER_ID_FLAT_POWER       : u8 = 12;
const PARAMETER_ID_LOCK_RANGE       : u8 = 13;

impl Into<u8> for Parameter {
    fn into(self) -> u8 {
        match self {
            Self::DelayCompensation   => PARAMETER_ID_DELAY_COMP,
            Self::StartupFrequency    => PARAMETER_ID_STARTUP_FREQ,
            Self::LockRange           => PARAMETER_ID_LOCK_RANGE,
            Self::RunMode             => PARAMETER_ID_RUN_MODE,
            Self::LockTime            => PARAMETER_ID_LOCK_TIME,
            Self::StartupTime         => PARAMETER_ID_STARTUP_TIME,
            Self::OnTime              => PARAMETER_ID_ON_TIME,
            Self::OffTime             => PARAMETER_ID_OFF_TIME,
            Self::RampStartPower      => PARAMETER_ID_RAMP_START,
            Self::RampEndPower        => PARAMETER_ID_RAMP_END,
            Self::MinLockCurrent      => PARAMETER_ID_MIN_LOCK_CURRENT,
            Self::CurrentLimit        => PARAMETER_ID_CURRENT_LIMIT,
            Self::FlatPower           => PARAMETER_ID_FLAT_POWER,
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
            PARAMETER_ID_OFF_TIME         => Self::OffTime,
            PARAMETER_ID_RAMP_START       => Self::RampStartPower,
            PARAMETER_ID_RAMP_END         => Self::RampEndPower,
            PARAMETER_ID_MIN_LOCK_CURRENT => Self::MinLockCurrent,
            PARAMETER_ID_CURRENT_LIMIT    => Self::CurrentLimit,
            PARAMETER_ID_FLAT_POWER       => Self::FlatPower,
            _ => return Err(())
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Statistic {
    MaxPrimaryCurrent,
    FeedbackFrequency,
}

const STATISTIC_ID_MAX_PRIMARY_CURRENT: u8 = 0;
const STATISTIC_ID_FEEDBACK_FREQUENCY: u8 = 1;

#[derive(Copy, Clone, Debug)]
pub enum StatisticValue {
    MaxPrimaryCurrentA(f32),
    FeedbackFrequencykHz(f32),
}

impl Into<(Statistic, u16)> for StatisticValue {
    fn into(self) -> (Statistic, u16) {
        match self {
            Self::MaxPrimaryCurrentA(current) => (Statistic::MaxPrimaryCurrent, (current * 32.0).clamp(0.0, 16383.0) as u16),
            Self::FeedbackFrequencykHz(frequency) => (Statistic::FeedbackFrequency, (frequency * 16.0).clamp(0.0, 16383.0) as u16),
        }
    }
}

impl TryFrom<(Statistic, u16)> for StatisticValue {
    type Error = ();
    fn try_from(x: (Statistic, u16)) -> Result<Self, Self::Error> {
        let (stat, value) = x;
        Ok(match stat {
            Statistic::MaxPrimaryCurrent => Self::MaxPrimaryCurrentA(value as f32 / 32.0),
            Statistic::FeedbackFrequency => Self::FeedbackFrequencykHz(value as f32 / 16.0),
        })
    }
}

impl Into<u8> for Statistic {
    fn into(self) -> u8 {
        match self {
            Self::MaxPrimaryCurrent => STATISTIC_ID_MAX_PRIMARY_CURRENT,
            Self::FeedbackFrequency => STATISTIC_ID_FEEDBACK_FREQUENCY,
        }
    }
}

impl TryFrom<u8> for Statistic {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        Ok(match value {
            STATISTIC_ID_MAX_PRIMARY_CURRENT => Self::MaxPrimaryCurrent,
            STATISTIC_ID_FEEDBACK_FREQUENCY => Self::FeedbackFrequency,
            _ => return Err(())
        })
    }
}

const MESSAGE_START_BIT: u8 = 0x80;

#[derive(Copy, Clone, Debug)]
pub enum ControllerMessage {
    SetDebugLed(bool),
    GetParam(Parameter),
    SetParam(ParameterValue),
    GetStat(Statistic),
    ResetStats,
    KeepAlive,
    Run,
    Stop,
    Ping(u32),
}

const CONTROLLER_MESSAGE_ID_SET_DEBUG_LED: u8 = 0;
const CONTROLLER_MESSAGE_ID_GET_PARAM: u8 = 1;
const CONTROLLER_MESSAGE_ID_SET_PARAM: u8 = 2;
const CONTROLLER_MESSAGE_ID_GET_STAT: u8 = 3;
const CONTROLLER_MESSAGE_ID_RESET_STATS: u8 = 4;
const CONTROLLER_MESSAGE_ID_KEEP_ALIVE: u8 = 5;
const CONTROLLER_MESSAGE_ID_RUN: u8 = 6;
const CONTROLLER_MESSAGE_ID_STOP: u8 = 7;
const CONTROLLER_MESSAGE_ID_PING: u8 = 0x7F;

impl ControllerMessage {
    pub fn try_send<const N: usize>(&self, buffer: &mut SerialBuffer<N>) -> bool {
        let length = match self {
            Self::SetDebugLed(..) => 2,
            Self::GetParam(..)    => 2,
            Self::SetParam(..)    => 4,
            Self::GetStat(..)     => 2,
            Self::ResetStats      => 1,
            Self::KeepAlive       => 1,
            Self::Run             => 1,
            Self::Stop            => 1,
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
                Self::SetParam(parameter_value) => {
                    let (param, value) = (*parameter_value).into();
                    buffer.push(CONTROLLER_MESSAGE_ID_SET_PARAM | MESSAGE_START_BIT);
                    buffer.push(param.into());
                    buffer.push(((value >> 0) & 0x7F) as u8);
                    buffer.push(((value >> 7) & 0x7F) as u8);
                },
                Self::GetStat(stat) => {
                    buffer.push(CONTROLLER_MESSAGE_ID_GET_STAT | MESSAGE_START_BIT);
                    buffer.push((*stat).into());
                },
                Self::ResetStats => {
                    buffer.push(CONTROLLER_MESSAGE_ID_RESET_STATS | MESSAGE_START_BIT);
                },
                Self::KeepAlive => {
                    buffer.push(CONTROLLER_MESSAGE_ID_KEEP_ALIVE | MESSAGE_START_BIT);
                },
                Self::Run => {
                    buffer.push(CONTROLLER_MESSAGE_ID_RUN | MESSAGE_START_BIT);
                },
                Self::Stop => {
                    buffer.push(CONTROLLER_MESSAGE_ID_STOP | MESSAGE_START_BIT);
                },
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
                CONTROLLER_MESSAGE_ID_RUN => 1,
                CONTROLLER_MESSAGE_ID_STOP => 1,
                CONTROLLER_MESSAGE_ID_KEEP_ALIVE => 1,
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
                        let value = 
                            ((rx_buffer.pop().unwrap() as u16) << 0) |
                            ((rx_buffer.pop().unwrap() as u16) << 7);
                        let param = Parameter::try_from(param_id)?;
                        let param_value = ParameterValue::try_from((param, value))?;
                        return Ok(Some(ControllerMessage::SetParam(param_value)));
                    },
                    CONTROLLER_MESSAGE_ID_GET_STAT => {
                        let param_id = rx_buffer.pop().unwrap();
                        return Ok(Some(ControllerMessage::GetStat(Statistic::try_from(param_id)?)));
                    },
                    CONTROLLER_MESSAGE_ID_RESET_STATS => {
                        return Ok(Some(ControllerMessage::ResetStats));
                    },
                    CONTROLLER_MESSAGE_ID_KEEP_ALIVE => {
                        return Ok(Some(ControllerMessage::KeepAlive));
                    },
                    CONTROLLER_MESSAGE_ID_RUN => {
                        return Ok(Some(ControllerMessage::Run));
                    },
                    CONTROLLER_MESSAGE_ID_STOP => {
                        return Ok(Some(ControllerMessage::Stop));
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
    GetParamResult(ParameterValue),
    GetStatResult(StatisticValue),
    Ping(u32),
    LockFailed,
    OcdTripped,
}

const REMOTE_MESSAGE_ID_GET_PARAM_RESULT: u8 = 0;
const REMOTE_MESSAGE_ID_GET_STAT_RESULT: u8 = 1;
const REMOTE_MESSAGE_ID_LOCK_FAILED: u8 = 2;
const REMOTE_MESSAGE_ID_OCD_TRIPPED: u8 = 3;
const REMOTE_MESSAGE_ID_PING: u8 = 0x7F;

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
            Self::GetParamResult(param_value) => {
                if tx_buffer.free_space() >= 4 {
                    tx_buffer.push(REMOTE_MESSAGE_ID_GET_PARAM_RESULT | MESSAGE_START_BIT);
                    let (param, value) = (*param_value).into();
                    tx_buffer.push((param).into());
                    tx_buffer.push(((value >>  0) & 0x7F) as u8);
                    tx_buffer.push(((value >>  7) & 0x7F) as u8);
                    true
                } else {
                    false
                }
            },
            Self::GetStatResult(stat_value) => {
                if tx_buffer.free_space() >= 4 {
                    tx_buffer.push(REMOTE_MESSAGE_ID_GET_STAT_RESULT | MESSAGE_START_BIT);
                    let (stat, value) = (*stat_value).into();
                    tx_buffer.push(stat.into());
                    tx_buffer.push(((value >>  0) & 0x7F) as u8);
                    tx_buffer.push(((value >>  7) & 0x7F) as u8);
                    true
                } else {
                    false
                }
            },
            Self::LockFailed => {
                if tx_buffer.free_space() >= 1 {
                    tx_buffer.push(REMOTE_MESSAGE_ID_LOCK_FAILED | MESSAGE_START_BIT);
                    true
                } else {
                    false
                }
            },
            Self::OcdTripped => {
                if tx_buffer.free_space() >= 1 {
                    tx_buffer.push(REMOTE_MESSAGE_ID_OCD_TRIPPED | MESSAGE_START_BIT);
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
                REMOTE_MESSAGE_ID_LOCK_FAILED      => 1,
                REMOTE_MESSAGE_ID_OCD_TRIPPED      => 1,
                _ => { rx_buffer.pop(); return Err(()) }
            };
            if rx_buffer.count() >= length {
                _ = rx_buffer.pop();
                match id {
                    REMOTE_MESSAGE_ID_GET_PARAM_RESULT => {
                        let param_id = rx_buffer.pop().unwrap();
                        let value = 
                            ((rx_buffer.pop().unwrap() as u16) << 0) |
                            ((rx_buffer.pop().unwrap() as u16) << 7);
                        let param = Parameter::try_from(param_id)?;
                        let param_value = ParameterValue::try_from((param, value))?;
                        return Ok(Some(RemoteMessage::GetParamResult(param_value)));
                    },
                    REMOTE_MESSAGE_ID_GET_STAT_RESULT => {
                        let stat = Statistic::try_from(rx_buffer.pop().unwrap())?;
                        let value =
                            ((rx_buffer.pop().unwrap() as u16) <<  0) |
                            ((rx_buffer.pop().unwrap() as u16) <<  7);
                        let stat_value = StatisticValue::try_from((stat, value))?;
                        Ok(Some(Self::GetStatResult(stat_value)))
                    },
                    REMOTE_MESSAGE_ID_PING => {
                        let seq = 
                            ((rx_buffer.pop().unwrap() as u32) <<  0) |
                            ((rx_buffer.pop().unwrap() as u32) <<  7) |
                            ((rx_buffer.pop().unwrap() as u32) << 14) |
                            ((rx_buffer.pop().unwrap() as u32) << 21);
                        Ok(Some(Self::Ping(seq)))
                    },
                    REMOTE_MESSAGE_ID_LOCK_FAILED => {
                        Ok(Some(Self::LockFailed))
                    },
                    REMOTE_MESSAGE_ID_OCD_TRIPPED => {
                        Ok(Some(Self::OcdTripped))
                    }
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
