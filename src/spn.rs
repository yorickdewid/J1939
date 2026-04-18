use crate::{slots, PDU_NOT_AVAILABLE};

//
// Time/Date
//
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimeDate {
    /// Year.
    pub year: i32,
    /// Month.
    pub month: u32,
    /// Day.
    pub day: u32,
    /// Hour.
    pub hour: u32,
    /// Minute.
    pub minute: u32,
    /// Second.
    pub second: u32,
    /// Local minute offset (SPN 1601), minutes relative to UTC. None = Not available.
    pub local_minute_offset: Option<i8>,
    /// Local hour offset (SPN 1602), hours relative to UTC. None = Not available.
    pub local_hour_offset: Option<i8>,
}

impl TimeDate {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 6, "TimeDate::from_pdu requires at least 6 bytes, got {}", pdu.len());
        let local_minute_offset = if pdu.len() >= 7 { Some(pdu[6]) } else { None }
            .and_then(slots::minute_offset::dec);
        let local_hour_offset = if pdu.len() >= 8 { Some(pdu[7]) } else { None }
            .and_then(slots::hour_offset::dec);
        Self {
            year: i32::from(pdu[5]) + 1985,
            month: u32::from(pdu[3]),
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            day: (f32::from(pdu[4]) * 0.25) as u32,
            hour: u32::from(pdu[2]),
            minute: u32::from(pdu[1]),
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            second: (f32::from(pdu[0]) * 0.25) as u32,
            local_minute_offset,
            local_hour_offset,
        }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            (self.second * 4) as u8,
            self.minute as u8,
            self.hour as u8,
            self.month as u8,
            (self.day * 4) as u8,
            (self.year - 1985) as u8,
            slots::minute_offset::enc(self.local_minute_offset),
            slots::hour_offset::enc(self.local_hour_offset),
        ]
    }

    #[cfg(feature = "chrono")]
    #[must_use]
    pub fn as_date_time(&self) -> chrono::LocalResult<chrono::DateTime<chrono::Utc>> {
        use chrono::TimeZone;

        chrono::Utc.with_ymd_and_hms(
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        )
    }

    #[cfg(feature = "chrono")]
    #[must_use]
    pub fn from_date_time(dt: &chrono::DateTime<chrono::Utc>) -> Self {
        use chrono::prelude::*;

        Self {
            year: dt.year(),
            month: dt.month(),
            day: dt.day(),
            hour: dt.hour(),
            minute: dt.minute(),
            second: dt.second(),
            local_minute_offset: None,
            local_hour_offset: None,
        }
    }
}

//
// Electronic Engine Controller 1
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EngineTorqueMode {
    NoRequest,
    AcceleratorPedal,
    CruiseControl,
    PTOGovernor,
    RoadSpeedGovernor,
    ASRControl,
    TransmissionControl,
    ABSControl,
    TorqueLimiting,
    HighSpeedGovernor,
    BrakingSystem,
    RemoteAccelerator,
    Other,
}

impl EngineTorqueMode {
    #[must_use]
    pub fn from_value(value: u8) -> Option<Self> {
        match value & 0b1111 {
            0b0000 => Some(Self::NoRequest),
            0b0001 => Some(Self::AcceleratorPedal),
            0b0010 => Some(Self::CruiseControl),
            0b0011 => Some(Self::PTOGovernor),
            0b0100 => Some(Self::RoadSpeedGovernor),
            0b0101 => Some(Self::ASRControl),
            0b0110 => Some(Self::TransmissionControl),
            0b0111 => Some(Self::ABSControl),
            0b1000 => Some(Self::TorqueLimiting),
            0b1001 => Some(Self::HighSpeedGovernor),
            0b1010 => Some(Self::BrakingSystem),
            0b1011 => Some(Self::RemoteAccelerator),
            0b1100..=0b1110 => Some(Self::Other),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_value(mode: Option<Self>) -> u8 {
        match mode {
            Some(Self::NoRequest) => 0b0000,
            Some(Self::AcceleratorPedal) => 0b0001,
            Some(Self::CruiseControl) => 0b0010,
            Some(Self::PTOGovernor) => 0b0011,
            Some(Self::RoadSpeedGovernor) => 0b0100,
            Some(Self::ASRControl) => 0b0101,
            Some(Self::TransmissionControl) => 0b0110,
            Some(Self::ABSControl) => 0b0111,
            Some(Self::TorqueLimiting) => 0b1000,
            Some(Self::HighSpeedGovernor) => 0b1001,
            Some(Self::BrakingSystem) => 0b1010,
            Some(Self::RemoteAccelerator) => 0b1011,
            Some(Self::Other) | None => 0b1111,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EngineStarterMode {
    StartNotRequested,
    StarterActiveGearNotEngaged,
    StarterActiveGearEngaged,
    StartFinished,
    StarterInhibitedEngineRunning,
    StarterInhibitedEngineNotReady,
    StarterInhibitedTransmissionInhibited,
    StarterInhibitedActiveImmobilizer,
    StarterInhibitedOverHeat,
    StarterInhibitedReasonUnknown,
    Error,
    Reserved,
}

impl EngineStarterMode {
    #[must_use]
    pub fn from_value(value: u8) -> Option<Self> {
        match value & 0b1111 {
            0b0000 => Some(Self::StartNotRequested),
            0b0001 => Some(Self::StarterActiveGearNotEngaged),
            0b0010 => Some(Self::StarterActiveGearEngaged),
            0b0011 => Some(Self::StartFinished),
            0b0100 => Some(Self::StarterInhibitedEngineRunning),
            0b0101 => Some(Self::StarterInhibitedEngineNotReady),
            0b0110 => Some(Self::StarterInhibitedTransmissionInhibited),
            0b0111 => Some(Self::StarterInhibitedActiveImmobilizer),
            0b1000 => Some(Self::StarterInhibitedOverHeat),
            0b1001..=0b1011 => Some(Self::Reserved),
            0b1100 => Some(Self::StarterInhibitedReasonUnknown),
            0b1101 | 0b1110 => Some(Self::Error),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_value(mode: Option<Self>) -> u8 {
        match mode {
            Some(Self::StartNotRequested) => 0b0000,
            Some(Self::StarterActiveGearNotEngaged) => 0b0001,
            Some(Self::StarterActiveGearEngaged) => 0b0010,
            Some(Self::StartFinished) => 0b0011,
            Some(Self::StarterInhibitedEngineRunning) => 0b0100,
            Some(Self::StarterInhibitedEngineNotReady) => 0b0101,
            Some(Self::StarterInhibitedTransmissionInhibited) => 0b0110,
            Some(Self::StarterInhibitedActiveImmobilizer) => 0b0111,
            Some(Self::StarterInhibitedOverHeat) => 0b1000,
            Some(Self::Reserved) => 0b1001,
            Some(Self::StarterInhibitedReasonUnknown) => 0b1100,
            Some(Self::Error) => 0b1101,
            None => 0b1111,
        }
    }
}

pub struct ElectronicEngineController1Message {
    /// Engine Torque Mode - SPN 899.
    pub engine_torque_mode: Option<EngineTorqueMode>,
    /// Driver's Demand Engine - Percent Torque.
    pub driver_demand: Option<u8>,
    /// Actual Engine - Percent Torque.
    pub actual_engine: Option<u8>,
    /// Engine Speed.
    pub rpm: Option<u16>,
    /// Source Address of Controlling Device for Engine Control - SPN 1483.
    pub source_addr: Option<u8>,
    /// Engine Starter Mode - SPN 1675.
    pub starter_mode: Option<EngineStarterMode>,
}

impl ElectronicEngineController1Message {
    /// # Panics
    /// Panics if `pdu` has fewer than 7 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 7, "ElectronicEngineController1Message::from_pdu requires at least 7 bytes, got {}", pdu.len());
        Self {
            engine_torque_mode: EngineTorqueMode::from_value(pdu[0]),
            driver_demand: slots::position_level2::dec(pdu[1]),
            actual_engine: slots::position_level2::dec(pdu[2]),
            rpm: slots::rotational_velocity::dec([pdu[3], pdu[4]]),
            source_addr: slots::source_address::dec(pdu[5]),
            starter_mode: EngineStarterMode::from_value(pdu[6]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            EngineTorqueMode::to_value(self.engine_torque_mode),
            slots::position_level2::enc(self.driver_demand),
            slots::position_level2::enc(self.actual_engine),
            slots::rotational_velocity::enc(self.rpm)[0],
            slots::rotational_velocity::enc(self.rpm)[1],
            slots::source_address::enc(self.source_addr),
            EngineStarterMode::to_value(self.starter_mode),
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for ElectronicEngineController1Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Torque mode: {:?}; Driver demand: {}%; Actual engine: {}%; RPM: {}; Starter mode: {:?}",
            self.engine_torque_mode,
            self.driver_demand.unwrap_or(0),
            self.actual_engine.unwrap_or(0),
            self.rpm.unwrap_or(0),
            self.starter_mode
        )
    }
}

//
// Electronic Engine Controller 2
//

pub struct ElectronicEngineController2Message {
    /// Switch signal which indicates the state of the accelerator pedal 1 low
    /// idle switch.
    pub accelerator_pedal1_low_idle_switch: Option<bool>,
    /// Switch signal which indicates whether the accelerator pedal kickdown
    /// switch is opened or closed.
    pub accelerator_pedal_kickdown_switch: Option<bool>,
    /// Status (active or not active) of the system used to limit maximum vehicle velocity.
    pub road_speed_limit_status: Option<bool>,
    /// The ratio of actual position of the analog engine speed/torque request input device
    /// (such as an accelerator pedal or throttle lever) to the maximum position of the input device.
    pub accelerator_pedal_position1: Option<u8>,
    /// The ratio of actual engine percent torque (indicated) to maximum indicated
    // torque available at the current engine speed, clipped to zero torque during engine braking.
    pub percent_load_at_current_speed: Option<u8>,
    /// The ratio of actual position of the remote analog engine speed/torque
    // request input device (such as an accelerator pedal or throttle lever) to the maximum position of the input device.
    pub remote_accelerator_pedal_position: Option<u8>,
}

impl ElectronicEngineController2Message {
    /// # Panics
    /// Panics if `pdu` has fewer than 4 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 4, "ElectronicEngineController2Message::from_pdu requires at least 4 bytes, got {}", pdu.len());
        Self {
            accelerator_pedal1_low_idle_switch: slots::bool_from_value(pdu[0]),
            accelerator_pedal_kickdown_switch: slots::bool_from_value(pdu[0] >> 2),
            road_speed_limit_status: slots::bool_from_value(pdu[0] >> 4),
            accelerator_pedal_position1: slots::position_level2::dec(pdu[1]),
            percent_load_at_current_speed: slots::position_level3::dec(pdu[2]),
            remote_accelerator_pedal_position: slots::position_level::dec(pdu[3]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::bool_to_value(self.accelerator_pedal1_low_idle_switch)
                | slots::bool_to_value(self.accelerator_pedal_kickdown_switch) << 2
                | slots::bool_to_value(self.road_speed_limit_status) << 4,
            slots::position_level2::enc(self.accelerator_pedal_position1),
            slots::position_level3::enc(self.percent_load_at_current_speed),
            slots::position_level::enc(self.remote_accelerator_pedal_position),
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for ElectronicEngineController2Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Accelerator pedal 1 low idle switch: {:?}; Accelerator pedal kickdown switch: {:?}; Road speed limit status: {:?}; Accelerator pedal position 1: {}%; Percent load at current speed: {}%; Remote accelerator pedal position: {}%",
            self.accelerator_pedal1_low_idle_switch,
            self.accelerator_pedal_kickdown_switch,
            self.road_speed_limit_status,
            self.accelerator_pedal_position1.unwrap_or(0),
            self.percent_load_at_current_speed.unwrap_or(0),
            self.remote_accelerator_pedal_position.unwrap_or(0)
        )
    }
}

//
// Electronic Engine Controller 3
//

pub struct ElectronicEngineController3Message {
    /// The calculated torque that indicates the amount of torque required by
    /// the basic engine itself added by the loss torque of accessories.
    pub nominal_friction_percent_torque: Option<u8>,
    /// An indication by the engine of the optimal operating speed of the engine
    /// for the current existing conditions. These conditions may include the torque generated to accommodate powertrain demands from the
    /// operator (via the accelerator pedal), cruise control, road speed limit governors, or ASR. Dynamic commands from functions such as
    /// smoke control or shift control are excluded from this calculation.
    pub engines_desired_operating_speed: Option<u16>,
    /// This byte is utilized in transmission gear
    /// selection routines and indicates the engine's preference of lower versus higher engine speeds should its desired speed not be achievable.
    pub engines_desired_operating_speed_asymmetry_adjustment: Option<u8>,
}

impl ElectronicEngineController3Message {
    /// # Panics
    /// Panics if `pdu` has fewer than 4 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 4, "ElectronicEngineController3Message::from_pdu requires at least 4 bytes, got {}", pdu.len());
        Self {
            nominal_friction_percent_torque: slots::position_level2::dec(pdu[0]),
            engines_desired_operating_speed: slots::rotational_velocity::dec([pdu[1], pdu[2]]),
            engines_desired_operating_speed_asymmetry_adjustment: slots::count::dec(pdu[3]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::position_level2::enc(self.nominal_friction_percent_torque),
            slots::rotational_velocity::enc(self.engines_desired_operating_speed)[0],
            slots::rotational_velocity::enc(self.engines_desired_operating_speed)[1],
            slots::count::enc(self.engines_desired_operating_speed_asymmetry_adjustment),
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for ElectronicEngineController3Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Nominal friction percent torque: {}%; Engines desired operating speed: {} RPM; Engines desired operating speed asymmetry adjustment: {}",
            self.nominal_friction_percent_torque.unwrap_or(0),
            self.engines_desired_operating_speed.unwrap_or(0),
            self.engines_desired_operating_speed_asymmetry_adjustment.unwrap_or(0)
        )
    }
}

//
// Torque Speed Control 1
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OverrideControlMode {
    OverrideDisabled,
    SpeedControl,
    TorqueControl,
    SpeedTorqueLimitControl,
}

impl OverrideControlMode {
    #[must_use]
    pub fn from_value(value: u8) -> Self {
        match value & 0b11 {
            0b00 => OverrideControlMode::OverrideDisabled,
            0b01 => OverrideControlMode::SpeedControl,
            0b10 => OverrideControlMode::TorqueControl,
            0b11 => OverrideControlMode::SpeedTorqueLimitControl,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn to_value(mode: Self) -> u8 {
        match mode {
            OverrideControlMode::OverrideDisabled => 0b00,
            OverrideControlMode::SpeedControl => 0b01,
            OverrideControlMode::TorqueControl => 0b10,
            OverrideControlMode::SpeedTorqueLimitControl => 0b11,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RequestedSpeedControlCondition {
    TransientOptimizedDriveLineDisengaged,
    StabilityOptimizedDriveLineDisengaged,
    StabilityOptimizedDriveLineEngaged1,
    StabilityOptimizedDriveLineEngaged2,
}

impl RequestedSpeedControlCondition {
    #[must_use]
    pub fn from_value(value: u8) -> Self {
        match value & 0b11 {
            0b00 => RequestedSpeedControlCondition::TransientOptimizedDriveLineDisengaged,
            0b01 => RequestedSpeedControlCondition::StabilityOptimizedDriveLineDisengaged,
            0b10 => RequestedSpeedControlCondition::StabilityOptimizedDriveLineEngaged1,
            0b11 => RequestedSpeedControlCondition::StabilityOptimizedDriveLineEngaged2,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn to_value(condition: Self) -> u8 {
        match condition {
            RequestedSpeedControlCondition::TransientOptimizedDriveLineDisengaged => 0b00,
            RequestedSpeedControlCondition::StabilityOptimizedDriveLineDisengaged => 0b01,
            RequestedSpeedControlCondition::StabilityOptimizedDriveLineEngaged1 => 0b10,
            RequestedSpeedControlCondition::StabilityOptimizedDriveLineEngaged2 => 0b11,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OverrideControlModePriority {
    HighestPriority,
    HighPriority,
    MediumPriority,
    LowPriority,
}

impl OverrideControlModePriority {
    #[must_use]
    pub fn from_value(value: u8) -> Self {
        match value & 0b11 {
            0b00 => OverrideControlModePriority::HighestPriority,
            0b01 => OverrideControlModePriority::HighPriority,
            0b10 => OverrideControlModePriority::MediumPriority,
            0b11 => OverrideControlModePriority::LowPriority,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn to_value(priority: Self) -> u8 {
        match priority {
            OverrideControlModePriority::HighestPriority => 0b00,
            OverrideControlModePriority::HighPriority => 0b01,
            OverrideControlModePriority::MediumPriority => 0b10,
            OverrideControlModePriority::LowPriority => 0b11,
        }
    }
}

pub struct TorqueSpeedControl1Message {
    /// Override control mode - SPN 695
    pub override_control_mode: OverrideControlMode,
    /// This mode tells the engine control system the governor characteristics that are desired during speed control.
    pub speed_control_condition: RequestedSpeedControlCondition,
    /// This field is used as an input to the engine or retarder to determine the
    /// priority of the Override Control Mode received in the Torque/Speed Control message (see PGN 0). The default is 11 (Low priority). It
    /// is not required to use the same priority during the entire override function. For example, the transmission can use priority 01 (High
    /// priority) during a shift, but can set the priority to 11 (Low priority) at the end of the shift to allow traction control to also interact with
    /// the torque limit of the engine.
    pub control_mode_priority: OverrideControlModePriority,
    /// Requested speed or speed limit - SPN 898
    pub speed: Option<u16>,
    /// Requested torque or torque limit - SPN 518
    pub torque: Option<u8>,
}

impl TorqueSpeedControl1Message {
    /// # Panics
    /// Panics if `pdu` has fewer than 4 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 4, "TorqueSpeedControl1Message::from_pdu requires at least 4 bytes, got {}", pdu.len());
        Self {
            override_control_mode: OverrideControlMode::from_value(pdu[0]),
            speed_control_condition: RequestedSpeedControlCondition::from_value(pdu[0] >> 2),
            control_mode_priority: OverrideControlModePriority::from_value(pdu[0] >> 4),
            speed: slots::rotational_velocity::dec([pdu[1], pdu[2]]),
            torque: slots::position_level2::dec(pdu[3]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            OverrideControlMode::to_value(self.override_control_mode)
                | RequestedSpeedControlCondition::to_value(self.speed_control_condition) << 2
                | OverrideControlModePriority::to_value(self.control_mode_priority) << 4,
            slots::rotational_velocity::enc(self.speed)[0],
            slots::rotational_velocity::enc(self.speed)[1],
            slots::position_level2::enc(self.torque),
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for TorqueSpeedControl1Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Override control mode: {:?}; Speed control condition: {:?}; Control mode priority: {:?}; Speed: {}; Torque: {}",
            self.override_control_mode,
            self.speed_control_condition,
            self.control_mode_priority,
            self.speed.unwrap_or(0),
            self.torque.unwrap_or(0)
        )
    }
}

//
// Ambient Conditions
//

pub struct AmbientConditionsMessage {
    /// Barometric pressure.
    pub barometric_pressure: Option<u8>,
    /// Cab interior temperature.
    pub cab_interior_temperature: Option<i16>,
    /// Ambient air temperature.
    pub ambient_air_temperature: Option<i16>,
    /// Air inlet temperature.
    pub air_inlet_temperature: Option<i8>,
    /// Road surface temperature.
    pub road_surface_temperature: Option<i16>,
}

impl AmbientConditionsMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "AmbientConditionsMessage::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            barometric_pressure: if pdu[0] == PDU_NOT_AVAILABLE {
                None
            } else {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Some((f32::from(pdu[0]) * 0.5) as u8)
            },
            cab_interior_temperature: slots::temperature::dec([pdu[1], pdu[2]]),
            ambient_air_temperature: slots::temperature::dec([pdu[3], pdu[4]]),
            air_inlet_temperature: slots::temperature2::dec(pdu[5]),
            road_surface_temperature: slots::temperature::dec([pdu[6], pdu[7]]),
        }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            if let Some(pressure) = self.barometric_pressure {
                (f32::from(pressure) * 2.0) as u8
            } else {
                PDU_NOT_AVAILABLE
            },
            slots::temperature::enc(self.cab_interior_temperature)[0],
            slots::temperature::enc(self.cab_interior_temperature)[1],
            slots::temperature::enc(self.ambient_air_temperature)[0],
            slots::temperature::enc(self.ambient_air_temperature)[1],
            slots::temperature2::enc(self.air_inlet_temperature),
            slots::temperature::enc(self.road_surface_temperature)[0],
            slots::temperature::enc(self.road_surface_temperature)[1],
        ]
    }
}

impl core::fmt::Display for AmbientConditionsMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Barometric pressure: {} kPa; Cab interior temperature: {}°C; Ambient air temperature: {}°C; Air inlet temperature: {}°C; Road surface temperature: {}°C",
            self.barometric_pressure.unwrap_or(0),
            self.cab_interior_temperature.unwrap_or(0),
            self.ambient_air_temperature.unwrap_or(0),
            self.air_inlet_temperature.unwrap_or(0),
            self.road_surface_temperature.unwrap_or(0)
        )
    }
}

//
// Vehicle Position
//

// TODO: Not tested
pub struct VehiclePositionMessage {
    /// Latitude.
    pub latitude: Option<f32>,
    /// Longitude.
    pub longitude: Option<f32>,
}

impl VehiclePositionMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "VehiclePositionMessage::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            latitude: if [pdu[0], pdu[1], pdu[2], pdu[3]] == [PDU_NOT_AVAILABLE; 4] {
                None
            } else {
                Some((i32::from_le_bytes([pdu[0], pdu[1], pdu[2], pdu[3]]) - 210) as f32 * 1e-7)
            },
            longitude: if [pdu[4], pdu[5], pdu[6], pdu[7]] == [PDU_NOT_AVAILABLE; 4] {
                None
            } else {
                Some((i32::from_le_bytes([pdu[4], pdu[5], pdu[6], pdu[7]]) - 210) as f32 * 1e-7)
            },
        }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_pdu(&self) -> [u8; 8] {
        let lat_bytes = self
            .latitude
            .map_or([PDU_NOT_AVAILABLE; 4], |v| {
                ((v * 1e7) as i32 + 210).to_le_bytes()
            });
        let lon_bytes = self
            .longitude
            .map_or([PDU_NOT_AVAILABLE; 4], |v| {
                ((v * 1e7) as i32 + 210).to_le_bytes()
            });

        [
            lat_bytes[0], lat_bytes[1], lat_bytes[2], lat_bytes[3],
            lon_bytes[0], lon_bytes[1], lon_bytes[2], lon_bytes[3],
        ]
    }
}

impl core::fmt::Display for VehiclePositionMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Latitude: {:?}; Longitude: {:?}",
            self.latitude.unwrap_or(0.0),
            self.longitude.unwrap_or(0.0)
        )
    }
}

//
// Fuel Economy
//

// TODO: Not tested
pub struct FuelEconomyMessage {
    /// Amount of fuel consumed by engine per unit of time.
    pub fuel_rate: Option<f32>,
    /// Current fuel economy at current vehicle velocity.
    pub instantaneous_fuel_economy: Option<f32>,
    /// Average of instantaneous fuel economy for that segment of vehicle operation of interest.
    pub average_fuel_economy: Option<f32>,
    /// The position of the valve used to regulate the supply of a fluid, usually air or fuel/air
    /// mixture, to an engine. 0% represents no supply and 100% is full supply.
    pub throttle_position: Option<u8>,
}

impl FuelEconomyMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 7 bytes.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 7, "FuelEconomyMessage::from_pdu requires at least 7 bytes, got {}", pdu.len());
        Self {
            fuel_rate: if [pdu[0], pdu[1]] == [PDU_NOT_AVAILABLE; 2] {
                None
            } else {
                Some((f32::from(u16::from_le_bytes([pdu[0], pdu[1]])) * 0.05).clamp(0.0, 3212.75))
            },
            instantaneous_fuel_economy: if [pdu[2], pdu[3]] == [PDU_NOT_AVAILABLE; 2] {
                None
            } else {
                Some(
                    (f32::from(u16::from_le_bytes([pdu[2], pdu[3]])) * (1.0 / 512.0)).clamp(0.0, 125.5),
                )
            },
            average_fuel_economy: if [pdu[4], pdu[5]] == [PDU_NOT_AVAILABLE; 2] {
                None
            } else {
                Some(
                    (f32::from(u16::from_le_bytes([pdu[4], pdu[5]])) * (1.0 / 512.0)).clamp(0.0, 125.5),
                )
            },
            throttle_position: slots::position_level::dec(pdu[6]),
        }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn to_pdu(&self) -> [u8; 8] {
        let fuel_rate_bytes = self
            .fuel_rate
            .map_or([PDU_NOT_AVAILABLE; 2], |v| ((v * 20.0) as u16).to_le_bytes());
        let inst_bytes = self
            .instantaneous_fuel_economy
            .map_or([PDU_NOT_AVAILABLE; 2], |v| ((v * 512.0) as u16).to_le_bytes());
        let avg_bytes = self
            .average_fuel_economy
            .map_or([PDU_NOT_AVAILABLE; 2], |v| ((v * 512.0) as u16).to_le_bytes());

        [
            fuel_rate_bytes[0],
            fuel_rate_bytes[1],
            inst_bytes[0],
            inst_bytes[1],
            avg_bytes[0],
            avg_bytes[1],
            slots::position_level::enc(self.throttle_position),
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for FuelEconomyMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Fuel rate: {} L/h; Instantaneous fuel economy: {} km/kg; Average fuel economy: {} km/kg; Throttle position: {}%",
            self.fuel_rate.unwrap_or(0.0),
            self.instantaneous_fuel_economy.unwrap_or(0.0),
            self.average_fuel_economy.unwrap_or(0.0),
            self.throttle_position.unwrap_or(0)
        )
    }
}

//
// Engine Fluid Level/Pressure 1
//

pub struct EngineFluidLevelPressure1Message {
    /// Gage pressure of fuel in system as delivered from supply pump to the injection pump.
    pub fuel_delivery_pressure: Option<u8>,
    /// Differential crankcase blow-by pressure as measured through a tube with a venturi.
    pub extended_crankcase_blow_by_pressure: Option<u8>,
    /// Ratio of current volume of engine sump oil to maximum required volume.
    pub engine_oil_level: Option<u8>,
    /// Gage pressure of oil in engine lubrication system as provided by oil pump.
    pub engine_oil_pressure: Option<u8>,
    /// Gage pressure inside engine crankcase.
    pub crankcase_pressure: Option<i16>,
    /// Gage pressure of liquid found in engine cooling system.
    pub coolant_pressure: Option<u8>,
    /// Ratio of volume of liquid found in engine cooling system to total cooling system volume. Typical
    /// monitoring location is in the coolant expansion tank.
    pub coolant_level: Option<u8>,
}

impl EngineFluidLevelPressure1Message {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "EngineFluidLevelPressure1Message::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            fuel_delivery_pressure: slots::pressure::dec(pdu[0]),
            extended_crankcase_blow_by_pressure: slots::pressure2::dec(pdu[1]),
            engine_oil_level: slots::position_level::dec(pdu[2]),
            engine_oil_pressure: slots::pressure::dec(pdu[3]),
            crankcase_pressure: slots::pressure4::dec([pdu[4], pdu[5]]),
            coolant_pressure: slots::pressure3::dec(pdu[6]),
            coolant_level: slots::position_level::dec(pdu[7]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::pressure::enc(self.fuel_delivery_pressure),
            slots::pressure2::enc(self.extended_crankcase_blow_by_pressure),
            slots::position_level::enc(self.engine_oil_level),
            slots::pressure::enc(self.engine_oil_pressure),
            slots::pressure4::enc(self.crankcase_pressure)[0],
            slots::pressure4::enc(self.crankcase_pressure)[1],
            slots::pressure3::enc(self.coolant_pressure),
            slots::position_level::enc(self.coolant_level),
        ]
    }
}

impl core::fmt::Display for EngineFluidLevelPressure1Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Fuel delivery pressure: {} kPa; Extended crankcase blow-by pressure: {} kPa; Engine oil level: {}%; Engine oil pressure: {} kPa; Crankcase pressure: {} kPa; Coolant pressure: {} kPa; Coolant level: {}%",
            self.fuel_delivery_pressure.unwrap_or(0),
            self.extended_crankcase_blow_by_pressure.unwrap_or(0),
            self.engine_oil_level.unwrap_or(0),
            self.engine_oil_pressure.unwrap_or(0),
            self.crankcase_pressure.unwrap_or(0),
            self.coolant_pressure.unwrap_or(0),
            self.coolant_level.unwrap_or(0)
        )
    }
}

//
// Fuel Consumption (Liquid)
//

pub struct FuelConsumptionMessage {
    /// Fuel consumed during all or part of a journey.
    pub trip_fuel: Option<u32>,
    /// Accumulated amount of fuel used during vehicle operation.
    pub total_fuel_used: Option<u32>,
}

impl FuelConsumptionMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "FuelConsumptionMessage::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            trip_fuel: slots::liquid_fuel_usage::dec([pdu[0], pdu[1], pdu[2], pdu[3]]),
            total_fuel_used: slots::liquid_fuel_usage::dec([pdu[4], pdu[5], pdu[6], pdu[7]]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::liquid_fuel_usage::enc(self.trip_fuel)[0],
            slots::liquid_fuel_usage::enc(self.trip_fuel)[1],
            slots::liquid_fuel_usage::enc(self.trip_fuel)[2],
            slots::liquid_fuel_usage::enc(self.trip_fuel)[3],
            slots::liquid_fuel_usage::enc(self.total_fuel_used)[0],
            slots::liquid_fuel_usage::enc(self.total_fuel_used)[1],
            slots::liquid_fuel_usage::enc(self.total_fuel_used)[2],
            slots::liquid_fuel_usage::enc(self.total_fuel_used)[3],
        ]
    }
}

impl core::fmt::Display for FuelConsumptionMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Trip fuel: {} L; Total fuel used: {} L",
            self.trip_fuel.unwrap_or(0),
            self.total_fuel_used.unwrap_or(0)
        )
    }
}

//
// Vehicle Distance
//

pub struct VehicleDistanceMessage {
    /// Distance traveled during all or part of a journey.
    pub trip_distance: Option<u32>,
    /// Accumulated distance traveled by vehicle during its operation.
    pub total_vehicle_distance: Option<u32>,
}

impl VehicleDistanceMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "VehicleDistanceMessage::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            trip_distance: slots::distance::dec([pdu[0], pdu[1], pdu[2], pdu[3]]),
            total_vehicle_distance: slots::distance::dec([pdu[4], pdu[5], pdu[6], pdu[7]]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::distance::enc(self.trip_distance)[0],
            slots::distance::enc(self.trip_distance)[1],
            slots::distance::enc(self.trip_distance)[2],
            slots::distance::enc(self.trip_distance)[3],
            slots::distance::enc(self.total_vehicle_distance)[0],
            slots::distance::enc(self.total_vehicle_distance)[1],
            slots::distance::enc(self.total_vehicle_distance)[2],
            slots::distance::enc(self.total_vehicle_distance)[3],
        ]
    }
}

impl core::fmt::Display for VehicleDistanceMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Trip distance: {} km; Total vehicle distance: {} km",
            self.trip_distance.unwrap_or(0),
            self.total_vehicle_distance.unwrap_or(0)
        )
    }
}

//
// ECU History
//

pub struct ECUHistoryMessage {
    /// Total distance accumulated over the life of the ECU. When the ECU is replaced this value
    /// shall be reset.
    pub total_ecu_distance: Option<u32>,
    /// Total time accumulated over the life of the ECU, from ignition switch ON to ignition
    /// switch OFF. When the ECU is replaced this value shall be reset.
    pub total_ecu_run_time: Option<u32>,
}

impl ECUHistoryMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "ECUHistoryMessage::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            total_ecu_distance: slots::distance::dec([pdu[0], pdu[1], pdu[2], pdu[3]]),
            total_ecu_run_time: slots::time::dec([pdu[4], pdu[5], pdu[6], pdu[7]]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::distance::enc(self.total_ecu_distance)[0],
            slots::distance::enc(self.total_ecu_distance)[1],
            slots::distance::enc(self.total_ecu_distance)[2],
            slots::distance::enc(self.total_ecu_distance)[3],
            slots::time::enc(self.total_ecu_run_time)[0],
            slots::time::enc(self.total_ecu_run_time)[1],
            slots::time::enc(self.total_ecu_run_time)[2],
            slots::time::enc(self.total_ecu_run_time)[3],
        ]
    }
}

//
// High Resolution Vehicle Distance (PGN 65217)
//

pub struct HighResolutionVehicleDistanceMessage {
    /// Accumulated distance traveled by the vehicle during its operation (meters).
    pub total_vehicle_distance_m: Option<u32>,
    /// Distance traveled during trip (meters).
    pub trip_distance_m: Option<u32>,
}

impl HighResolutionVehicleDistanceMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "HighResolutionVehicleDistanceMessage::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            total_vehicle_distance_m: slots::distance5m::dec([pdu[0], pdu[1], pdu[2], pdu[3]]),
            trip_distance_m: slots::distance5m::dec([pdu[4], pdu[5], pdu[6], pdu[7]]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::distance5m::enc(self.total_vehicle_distance_m)[0],
            slots::distance5m::enc(self.total_vehicle_distance_m)[1],
            slots::distance5m::enc(self.total_vehicle_distance_m)[2],
            slots::distance5m::enc(self.total_vehicle_distance_m)[3],
            slots::distance5m::enc(self.trip_distance_m)[0],
            slots::distance5m::enc(self.trip_distance_m)[1],
            slots::distance5m::enc(self.trip_distance_m)[2],
            slots::distance5m::enc(self.trip_distance_m)[3],
        ]
    }
}

impl core::fmt::Display for HighResolutionVehicleDistanceMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Trip distance (m): {}; Total vehicle distance (m): {}",
            self.trip_distance_m.unwrap_or(0),
            self.total_vehicle_distance_m.unwrap_or(0)
        )
    }
}

//
// Tachograph (PGN 65132)
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DriverWorkingState {
    RestSleeping,
    DriverAvailableShortBreak,
    Work,
    Drive,
}

impl DriverWorkingState {
    #[must_use]
    pub fn from_value(value: u8) -> Option<Self> {
        match value & 0b111 {
            0b000 => Some(Self::RestSleeping),
            0b001 => Some(Self::DriverAvailableShortBreak),
            0b010 => Some(Self::Work),
            0b011 => Some(Self::Drive),
            // 100,101 = reserved; 110 = error; 111 = N/A
            _ => None,
        }
    }

    #[must_use]
    pub fn to_value(v: Option<Self>) -> u8 {
        match v {
            Some(Self::RestSleeping) => 0b000,
            Some(Self::DriverAvailableShortBreak) => 0b001,
            Some(Self::Work) => 0b010,
            Some(Self::Drive) => 0b011,
            None => 0b111, // Not available
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DriverTimeRelatedStates {
    NormalNoLimits,
    Limit1_15minBefore4_5h,
    Limit2_4_5hReached,
    Limit3_15minBefore9h,
    Limit4_9hReached,
    Limit5_15minBefore16h,
    Limit6_16hReached,
    Other,
}

impl DriverTimeRelatedStates {
    #[must_use]
    pub fn from_value(value: u8) -> Option<Self> {
        match value & 0b1111 {
            0b0000 => Some(Self::NormalNoLimits),
            0b0001 => Some(Self::Limit1_15minBefore4_5h),
            0b0010 => Some(Self::Limit2_4_5hReached),
            0b0011 => Some(Self::Limit3_15minBefore9h),
            0b0100 => Some(Self::Limit4_9hReached),
            0b0101 => Some(Self::Limit5_15minBefore16h),
            0b0110 => Some(Self::Limit6_16hReached),
            0b1101 => Some(Self::Other),
            // 0111..1100 reserved; 1110 Error; 1111 N/A
            _ => None,
        }
    }

    #[must_use]
    pub fn to_value(v: Option<Self>) -> u8 {
        match v {
            Some(Self::NormalNoLimits) => 0b0000,
            Some(Self::Limit1_15minBefore4_5h) => 0b0001,
            Some(Self::Limit2_4_5hReached) => 0b0010,
            Some(Self::Limit3_15minBefore9h) => 0b0011,
            Some(Self::Limit4_9hReached) => 0b0100,
            Some(Self::Limit5_15minBefore16h) => 0b0101,
            Some(Self::Limit6_16hReached) => 0b0110,
            Some(Self::Other) => 0b1101,
            None => 0b1111, // Not available
        }
    }
}

pub struct TachographMessage {
    pub driver1_working_state: Option<DriverWorkingState>, // SPN 1612 (3 bits) at 1.1
    pub driver2_working_state: Option<DriverWorkingState>, // SPN 1613 (3 bits) at 1.4
    pub vehicle_motion: Option<bool>,                     // SPN 1611 (2 bits) at 1.7
    pub driver1_time_states: Option<DriverTimeRelatedStates>, // SPN 1617 (4 bits) at 2.1
    pub driver1_card_present: Option<bool>,               // SPN 1615 (2 bits) at 2.5
    pub vehicle_overspeed: Option<bool>,                  // SPN 1614 (2 bits) at 2.7
    pub driver2_time_states: Option<DriverTimeRelatedStates>, // SPN 1618 (4 bits) at 3.1
    pub driver2_card_present: Option<bool>,               // SPN 1616 (2 bits) at 3.5
    pub system_event: Option<bool>,                       // SPN 1622 (2 bits) at 4.1
    pub handling_information: Option<bool>,               // SPN 1621 (2 bits) at 4.3
    pub tachograph_performance: Option<bool>,             // SPN 1620 (2 bits) at 4.5
    pub direction_indicator: Option<bool>,                // SPN 1619 (2 bits) at 4.7
    pub tachograph_output_shaft_speed: Option<u16>,       // SPN 1623 bytes 5-6 SAEvr01
    pub tachograph_vehicle_speed: Option<u16>,            // SPN 1624 bytes 7-8 SAEvl02
}

impl TachographMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "TachographMessage::from_pdu requires at least 8 bytes, got {}", pdu.len());
        let b1 = pdu[0];
        let b2 = pdu[1];
        let b3 = pdu[2];
        let b4 = pdu[3];
        Self {
            driver1_working_state: DriverWorkingState::from_value(b1 & 0b0000_0111),
            driver2_working_state: DriverWorkingState::from_value((b1 >> 3) & 0b0000_0111),
            vehicle_motion: slots::bool_from_value((b1 >> 6) & 0b0000_0011),
            driver1_time_states: DriverTimeRelatedStates::from_value(b2 & 0b0000_1111),
            driver1_card_present: slots::bool_from_value((b2 >> 4) & 0b0000_0011),
            vehicle_overspeed: slots::bool_from_value((b2 >> 6) & 0b0000_0011),
            driver2_time_states: DriverTimeRelatedStates::from_value(b3 & 0b0000_1111),
            driver2_card_present: slots::bool_from_value((b3 >> 4) & 0b0000_0011),
            system_event: slots::bool_from_value(b4 & 0b0000_0011),
            handling_information: slots::bool_from_value((b4 >> 2) & 0b0000_0011),
            tachograph_performance: slots::bool_from_value((b4 >> 4) & 0b0000_0011),
            direction_indicator: slots::bool_from_value((b4 >> 6) & 0b0000_0011),
            tachograph_output_shaft_speed: slots::rotational_velocity::dec([pdu[4], pdu[5]]),
            tachograph_vehicle_speed: slots::velocity_linear2::dec([pdu[6], pdu[7]]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        let mut b1 = 0u8;
        b1 |= DriverWorkingState::to_value(self.driver1_working_state) & 0b0000_0111; // 1.1
        b1 |= (DriverWorkingState::to_value(self.driver2_working_state) & 0b0000_0111) << 3; // 1.4
        b1 |= (slots::bool_to_value(self.vehicle_motion) & 0b0000_0011) << 6; // 1.7

        let mut b2 = 0u8;
        b2 |= DriverTimeRelatedStates::to_value(self.driver1_time_states) & 0b0000_1111; // 2.1
        b2 |= (slots::bool_to_value(self.driver1_card_present) & 0b0000_0011) << 4; // 2.5
        b2 |= (slots::bool_to_value(self.vehicle_overspeed) & 0b0000_0011) << 6; // 2.7

        let mut b3 = 0u8;
        b3 |= DriverTimeRelatedStates::to_value(self.driver2_time_states) & 0b0000_1111; // 3.1
        b3 |= (slots::bool_to_value(self.driver2_card_present) & 0b0000_0011) << 4; // 3.5
        // bits 6-7 reserved: set to 'Not available' (11)
        b3 |= 0b11 << 6;

        let mut b4 = 0u8;
        b4 |= slots::bool_to_value(self.system_event) & 0b0000_0011; // 4.1
        b4 |= (slots::bool_to_value(self.handling_information) & 0b0000_0011) << 2; // 4.3
        b4 |= (slots::bool_to_value(self.tachograph_performance) & 0b0000_0011) << 4; // 4.5
        b4 |= (slots::bool_to_value(self.direction_indicator) & 0b0000_0011) << 6; // 4.7

        [
            b1,
            b2,
            b3,
            b4,
            slots::rotational_velocity::enc(self.tachograph_output_shaft_speed)[0],
            slots::rotational_velocity::enc(self.tachograph_output_shaft_speed)[1],
            slots::velocity_linear2::enc(self.tachograph_vehicle_speed)[0],
            slots::velocity_linear2::enc(self.tachograph_vehicle_speed)[1],
        ]
    }
}

impl core::fmt::Display for TachographMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "D1: {:?}, D2: {:?}, motion: {:?}, overspeed: {:?}, out_shaft_rpm: {:?}, veh_speed: {:?}",
            self.driver1_working_state,
            self.driver2_working_state,
            self.vehicle_motion,
            self.vehicle_overspeed,
            self.tachograph_output_shaft_speed,
            self.tachograph_vehicle_speed
        )
    }
}

impl core::fmt::Display for ECUHistoryMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Total ECU distance: {} km; Total ECU run time: {} s",
            self.total_ecu_distance.unwrap_or(0),
            self.total_ecu_run_time.unwrap_or(0)
        )
    }
}

//
// Cab Illumination Message
//

pub struct CabIlluminationMessage {
    /// Commanded backlight brightness level for all cab displays.
    pub illumination_brightness_percent: Option<u8>,
}

impl CabIlluminationMessage {
    /// # Panics
    /// Panics if `pdu` is empty.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(!pdu.is_empty(), "CabIlluminationMessage::from_pdu requires at least 1 byte, got 0");
        Self {
            illumination_brightness_percent: slots::position_level::dec(pdu[0]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::position_level::enc(self.illumination_brightness_percent),
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for CabIlluminationMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Illumination brightness percent: {}%",
            self.illumination_brightness_percent.unwrap_or(0)
        )
    }
}

//
// Fan Drive
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FanDriveState {
    FanOff,
    EngineSystemGeneral,
    ExcessiveEngineAirTemperature,
    ExcessiveEngineOilTemperature,
    ExcessiveEngineCoolantTemperature,
    ExcessiveTransmissionOilTemperature,
    ExcessiveHydraulicOilTemperature,
    DefaultOperation,
    NotDefined,
    ManualControl,
    TransmissionRetarder,
    ACSystem,
    Timer,
    EngineBrake,
    Other,
}

impl FanDriveState {
    #[must_use]
    pub fn from_value(value: u8) -> Option<Self> {
        match value & 0b1111 {
            0b0000 => Some(Self::FanOff),
            0b0001 => Some(Self::EngineSystemGeneral),
            0b0010 => Some(Self::ExcessiveEngineAirTemperature),
            0b0011 => Some(Self::ExcessiveEngineOilTemperature),
            0b0100 => Some(Self::ExcessiveEngineCoolantTemperature),
            0b0101 => Some(Self::ExcessiveTransmissionOilTemperature),
            0b0110 => Some(Self::ExcessiveHydraulicOilTemperature),
            0b0111 => Some(Self::DefaultOperation),
            0b1000 => Some(Self::NotDefined),
            0b1001 => Some(Self::ManualControl),
            0b1010 => Some(Self::TransmissionRetarder),
            0b1011 => Some(Self::ACSystem),
            0b1100 => Some(Self::Timer),
            0b1101 => Some(Self::EngineBrake),
            0b1110 => Some(Self::Other),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_value(mode: Option<Self>) -> u8 {
        match mode {
            Some(Self::FanOff) => 0b0000,
            Some(Self::EngineSystemGeneral) => 0b0001,
            Some(Self::ExcessiveEngineAirTemperature) => 0b0010,
            Some(Self::ExcessiveEngineOilTemperature) => 0b0011,
            Some(Self::ExcessiveEngineCoolantTemperature) => 0b0100,
            Some(Self::ExcessiveTransmissionOilTemperature) => 0b0101,
            Some(Self::ExcessiveHydraulicOilTemperature) => 0b0110,
            Some(Self::DefaultOperation) => 0b0111,
            Some(Self::NotDefined) => 0b1000,
            Some(Self::ManualControl) => 0b1001,
            Some(Self::TransmissionRetarder) => 0b1010,
            Some(Self::ACSystem) => 0b1011,
            Some(Self::Timer) => 0b1100,
            Some(Self::EngineBrake) => 0b1101,
            Some(Self::Other) => 0b1110,
            None => PDU_NOT_AVAILABLE,
        }
    }
}

pub struct FanDriveMessage {
    /// Estimated fan speed as a ratio of the fan drive (current speed) to the fully
    /// engaged fan drive (maximum fan speed). A two state fan (off/on) will use 0% and 100% respectively.
    pub estimated_percent_fan_speed: Option<u8>,
    /// This parameter is used to indicate the current state or mode of operation by the fan drive.
    pub fan_drive_state: Option<FanDriveState>,
    /// The speed of the fan associated with engine coolant system.
    pub fan_speed: Option<u16>,
}

impl FanDriveMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 4 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 4, "FanDriveMessage::from_pdu requires at least 4 bytes, got {}", pdu.len());
        Self {
            estimated_percent_fan_speed: slots::position_level::dec(pdu[0]),
            fan_drive_state: FanDriveState::from_value(pdu[1]),
            fan_speed: slots::rotational_velocity::dec([pdu[2], pdu[3]]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::position_level::enc(self.estimated_percent_fan_speed),
            FanDriveState::to_value(self.fan_drive_state),
            slots::rotational_velocity::enc(self.fan_speed)[0],
            slots::rotational_velocity::enc(self.fan_speed)[1],
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for FanDriveMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Estimated percent fan speed: {}%; Fan drive state: {:?}; Fan speed: {} RPM",
            self.estimated_percent_fan_speed.unwrap_or(0),
            self.fan_drive_state.unwrap_or(FanDriveState::FanOff),
            self.fan_speed.unwrap_or(0)
        )
    }
}

//
// Shutdown
//

pub struct ShutdownMessage {
    pub idle_shutdown_has_shutdown_engine: Option<bool>,
    pub idle_shutdown_driver_alert_mode: Option<bool>,
    pub idle_shutdown_timer_override: Option<bool>,
    pub idle_shutdown_timer_state: Option<bool>,
    pub idle_shutdown_timer_function: Option<bool>,
    pub ac_high_pressure_fan_switch: Option<bool>,
    pub refrigerant_low_pressure_switch: Option<bool>,
    pub refrigerant_high_pressure_switch: Option<bool>,
    pub wait_to_start_lamp: Option<bool>,
    pub engine_protection_system_has_shutdown_engine: Option<bool>,
    pub engine_protection_system_approaching_shutdown: Option<bool>,
    pub engine_protection_system_timer_override: Option<bool>,
    pub engine_protection_system_timer_state: Option<bool>,
    pub engine_protection_system_configuration: Option<bool>,
}

impl ShutdownMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 6 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 6, "ShutdownMessage::from_pdu requires at least 6 bytes, got {}", pdu.len());
        Self {
            idle_shutdown_has_shutdown_engine: slots::bool_from_value(pdu[0]),
            idle_shutdown_driver_alert_mode: slots::bool_from_value(pdu[0] >> 2),
            idle_shutdown_timer_override: slots::bool_from_value(pdu[0] >> 4),
            idle_shutdown_timer_state: slots::bool_from_value(pdu[0] >> 6),
            idle_shutdown_timer_function: slots::bool_from_value(pdu[1] >> 6),
            ac_high_pressure_fan_switch: slots::bool_from_value(pdu[2]),
            refrigerant_low_pressure_switch: slots::bool_from_value(pdu[2] >> 2),
            refrigerant_high_pressure_switch: slots::bool_from_value(pdu[2] >> 4),
            wait_to_start_lamp: slots::bool_from_value(pdu[3]),
            engine_protection_system_has_shutdown_engine: slots::bool_from_value(pdu[4]),
            engine_protection_system_approaching_shutdown: slots::bool_from_value(pdu[4] >> 2),
            engine_protection_system_timer_override: slots::bool_from_value(pdu[4] >> 4),
            engine_protection_system_timer_state: slots::bool_from_value(pdu[4] >> 6),
            engine_protection_system_configuration: slots::bool_from_value(pdu[5] >> 6),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::bool_to_value(self.idle_shutdown_has_shutdown_engine)
                | slots::bool_to_value(self.idle_shutdown_driver_alert_mode) << 2
                | slots::bool_to_value(self.idle_shutdown_timer_override) << 4
                | slots::bool_to_value(self.idle_shutdown_timer_state) << 6,
            slots::bool_to_value(self.idle_shutdown_timer_function) << 6,
            slots::bool_to_value(self.ac_high_pressure_fan_switch)
                | slots::bool_to_value(self.refrigerant_low_pressure_switch) << 2
                | slots::bool_to_value(self.refrigerant_high_pressure_switch) << 4,
            slots::bool_to_value(self.wait_to_start_lamp),
            slots::bool_to_value(self.engine_protection_system_has_shutdown_engine)
                | slots::bool_to_value(self.engine_protection_system_approaching_shutdown) << 2
                | slots::bool_to_value(self.engine_protection_system_timer_override) << 4
                | slots::bool_to_value(self.engine_protection_system_timer_state) << 6,
            slots::bool_to_value(self.engine_protection_system_configuration) << 6,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for ShutdownMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Idle shutdown has shutdown engine: {:?}; Idle shutdown driver alert mode: {:?}; Idle shutdown timer override: {:?}; Idle shutdown timer state: {:?}; Idle shutdown timer function: {:?}; AC high pressure fan switch: {:?}; Refrigerant low pressure switch: {:?}; Refrigerant high pressure switch: {:?}; Wait to start lamp: {:?}; Engine protection system has shutdown engine: {:?}; Engine protection system approaching shutdown: {:?}; Engine protection system timer override: {:?}; Engine protection system timer state: {:?}; Engine protection system configuration: {:?}",
            self.idle_shutdown_has_shutdown_engine,
            self.idle_shutdown_driver_alert_mode,
            self.idle_shutdown_timer_override,
            self.idle_shutdown_timer_state,
            self.idle_shutdown_timer_function,
            self.ac_high_pressure_fan_switch,
            self.refrigerant_low_pressure_switch,
            self.refrigerant_high_pressure_switch,
            self.wait_to_start_lamp,
            self.engine_protection_system_has_shutdown_engine,
            self.engine_protection_system_approaching_shutdown,
            self.engine_protection_system_timer_override,
            self.engine_protection_system_timer_state,
            self.engine_protection_system_configuration
        )
    }
}

//
// Power Takeoff Information
//

pub struct PowerTakeoffInformationMessage {
    /// Temperature of lubricant in device used to transmit engine power to auxiliary equipment.
    pub power_takeoff_oil_temperature: Option<i8>,
    /// Rotational velocity of device used to transmit engine power to auxiliary equipment.
    pub power_takeoff_speed: Option<u16>,
    /// Rotational velocity selected by operator for device used to transmit engine power to
    /// auxiliary equipment.
    pub power_takeoff_set_speed: Option<u16>,
    /// Switch signal which indicates that the PTO toggle switch is in the enabled (ON) position and
    /// therefore it is possible to manage the PTO control function.
    pub pto_enable_switch: Option<bool>,
    /// Switch signal which indicates that the remote
    /// PTO toggle switch is in the enabled (ON) position. If the toggle switch is enabled and other conditions are satisfied then the remote
    /// PTO control feature is activated and the PTO will control at the preprogrammed speed.
    pub remote_pto_preprogrammed_speed_control_switch: Option<bool>,
    /// Switch signal which indicates that the remote PTO toggle
    /// switch is in the enabled (ON) position. If the toggle switch is enabled and other conditions are satisfied then the remote PTO control
    /// feature is activated and the PTO will control at a variable speed.
    pub remote_pto_variable_speed_control_switch: Option<bool>,
    /// Switch signal of the PTO control activator which indicates that the activator is in the position "set".
    pub pto_set_switch: Option<bool>,
    /// Switch signal of the PTO control activator which indicates that the activator is in the position "coast/decelerate".
    pub pto_coast_decelerate_switch: Option<bool>,
    /// Switch signal of the PTO control activator which indicates that the activator is in the position "resume".
    pub pto_resume_switch: Option<bool>,
    /// Switch signal of the PTO control activator which indicates that the activator is in the position "accelerate".
    pub pto_accelerate_switch: Option<bool>,
}

impl PowerTakeoffInformationMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 7 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 7, "PowerTakeoffInformationMessage::from_pdu requires at least 7 bytes, got {}", pdu.len());
        Self {
            power_takeoff_oil_temperature: slots::temperature2::dec(pdu[0]),
            power_takeoff_speed: slots::rotational_velocity::dec([pdu[1], pdu[2]]),
            power_takeoff_set_speed: slots::rotational_velocity::dec([pdu[3], pdu[4]]),
            pto_enable_switch: slots::bool_from_value(pdu[5]),
            remote_pto_preprogrammed_speed_control_switch: slots::bool_from_value(pdu[5] >> 2),
            remote_pto_variable_speed_control_switch: slots::bool_from_value(pdu[5] >> 4),
            pto_set_switch: slots::bool_from_value(pdu[6]),
            pto_coast_decelerate_switch: slots::bool_from_value(pdu[6] >> 2),
            pto_resume_switch: slots::bool_from_value(pdu[6] >> 4),
            pto_accelerate_switch: slots::bool_from_value(pdu[6] >> 6),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::temperature2::enc(self.power_takeoff_oil_temperature),
            slots::rotational_velocity::enc(self.power_takeoff_speed)[0],
            slots::rotational_velocity::enc(self.power_takeoff_speed)[1],
            slots::rotational_velocity::enc(self.power_takeoff_set_speed)[0],
            slots::rotational_velocity::enc(self.power_takeoff_set_speed)[1],
            slots::bool_to_value(self.pto_enable_switch)
                | slots::bool_to_value(self.remote_pto_preprogrammed_speed_control_switch) << 2
                | slots::bool_to_value(self.remote_pto_variable_speed_control_switch) << 4,
            slots::bool_to_value(self.pto_set_switch)
                | slots::bool_to_value(self.pto_coast_decelerate_switch) << 2
                | slots::bool_to_value(self.pto_resume_switch) << 4
                | slots::bool_to_value(self.pto_accelerate_switch) << 6,
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for PowerTakeoffInformationMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Power takeoff oil temperature: {}°C; Power takeoff speed: {} RPM; Power takeoff set speed: {} RPM; PTO enable switch: {:?}; Remote PTO preprogrammed speed control switch: {:?}; Remote PTO variable speed control switch: {:?}; PTO set switch: {:?}; PTO coast/decelerate switch: {:?}; PTO resume switch: {:?}; PTO accelerate switch: {:?}",
            self.power_takeoff_oil_temperature.unwrap_or(0),
            self.power_takeoff_speed.unwrap_or(0),
            self.power_takeoff_set_speed.unwrap_or(0),
            self.pto_enable_switch,
            self.remote_pto_preprogrammed_speed_control_switch,
            self.remote_pto_variable_speed_control_switch,
            self.pto_set_switch,
            self.pto_coast_decelerate_switch,
            self.pto_resume_switch,
            self.pto_accelerate_switch
        )
    }
}

//
// Engine Temperature 1
//

pub struct EngineTemperature1Message {
    /// Temperature of liquid found in engine cooling system.
    pub engine_coolant_temperature: Option<i8>,
    /// Temperature of fuel entering injectors.
    pub fuel_temperature: Option<i8>,
    /// Temperature of the engine lubricant.
    pub engine_oil_temperature: Option<i16>,
    /// Temperature of the turbocharger lubricant.
    pub turbo_oil_temperature: Option<i16>,
    /// Temperature of liquid found in the intercooler located after the turbocharger.
    pub engine_intercooler_temperature: Option<i8>,
    /// The current position of the thermostat used to regulate the
    /// temperature of the engine intercooler. A value of 0% represents the thermostat being completely closed and 100% represents the
    /// thermostat being completely open.
    pub engine_intercooler_thermostat_opening: Option<u8>,
}

impl EngineTemperature1Message {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "EngineTemperature1Message::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            engine_coolant_temperature: slots::temperature2::dec(pdu[0]),
            fuel_temperature: slots::temperature2::dec(pdu[1]),
            engine_oil_temperature: slots::temperature::dec([pdu[2], pdu[3]]),
            turbo_oil_temperature: slots::temperature::dec([pdu[4], pdu[5]]),
            engine_intercooler_temperature: slots::temperature2::dec(pdu[6]),
            engine_intercooler_thermostat_opening: slots::position_level::dec(pdu[7]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::temperature2::enc(self.engine_coolant_temperature),
            slots::temperature2::enc(self.fuel_temperature),
            slots::temperature::enc(self.engine_oil_temperature)[0],
            slots::temperature::enc(self.engine_oil_temperature)[1],
            slots::temperature::enc(self.turbo_oil_temperature)[0],
            slots::temperature::enc(self.turbo_oil_temperature)[1],
            slots::temperature2::enc(self.engine_intercooler_temperature),
            slots::position_level::enc(self.engine_intercooler_thermostat_opening),
        ]
    }
}

impl core::fmt::Display for EngineTemperature1Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Engine coolant temperature: {}°C; Fuel temperature: {}°C; Engine oil temperature: {}°C; Turbo oil temperature: {}°C; Engine intercooler temperature: {}°C; Engine intercooler thermostat opening: {}%",
            self.engine_coolant_temperature.unwrap_or(0),
            self.fuel_temperature.unwrap_or(0),
            self.engine_oil_temperature.unwrap_or(0),
            self.turbo_oil_temperature.unwrap_or(0),
            self.engine_intercooler_temperature.unwrap_or(0),
            self.engine_intercooler_thermostat_opening.unwrap_or(0)
        )
    }
}

//
// Inlet/Exhaust Conditions 1
//

pub struct InletExhaustConditions1Message {
    /// Exhaust back pressure as a result of particle accumulation on filter media placed in the exhaust stream.
    pub particulate_trap_inlet_pressure: Option<u8>,
    /// Gage pressure of air measured downstream on the compressor discharge side of the turbocharger.
    /// See also SPNs 1127-1130 for alternate range and resolution. If there is one boost pressure to report and this range and resolution is
    /// adequate, this parameter should be used.
    pub boost_pressure: Option<u8>,
    /// Temperature of pre-combustion air found in intake manifold of engine air supply system.
    pub intake_manifold_temperature: Option<i8>,
    /// Absolute air pressure at inlet to intake manifold or air box.
    pub air_inlet_pressure: Option<u8>,
    /// Change in engine air system pressure, measured across the filter, due to the
    /// filter and any accumulation of solid foreign matter on or in the filter. This is the measurement of the first filter in a multiple air filter
    /// system. In a single air filter application, this is the only SPN used. Filter numbering follows the guidelines noted in section, Naming
    /// Convention For Engine Parameters.
    pub air_filter_differential_pressure: Option<u8>,
    /// Temperature of combustion byproducts leaving the engine. See SPNs 2433 and
    /// 2434 for engines with more than one exhause gas temperature measurement.
    pub exhaust_gas_temperature: Option<i16>,
    /// Change in coolant pressure, measured across the filter, due to the filter
    /// and any accumulation of solid or semisolid matter on or in the filter.
    pub coolant_filter_differential_pressure: Option<u8>,
}

impl InletExhaustConditions1Message {
    /// # Panics
    /// Panics if `pdu` has fewer than 7 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 7, "InletExhaustConditions1Message::from_pdu requires at least 7 bytes, got {}", pdu.len());
        Self {
            particulate_trap_inlet_pressure: None,
            boost_pressure: slots::pressure3::dec(pdu[1]),
            intake_manifold_temperature: slots::temperature2::dec(pdu[2]),
            air_inlet_pressure: slots::pressure3::dec(pdu[3]),
            air_filter_differential_pressure: slots::pressure2::dec(pdu[4]),
            exhaust_gas_temperature: slots::temperature::dec([pdu[5], pdu[6]]),
            coolant_filter_differential_pressure: None,
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            PDU_NOT_AVAILABLE,
            slots::pressure3::enc(self.boost_pressure),
            slots::temperature2::enc(self.intake_manifold_temperature),
            slots::pressure3::enc(self.air_inlet_pressure),
            slots::pressure2::enc(self.air_filter_differential_pressure),
            slots::temperature::enc(self.exhaust_gas_temperature)[0],
            slots::temperature::enc(self.exhaust_gas_temperature)[1],
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for InletExhaustConditions1Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Particulate trap inlet pressure: {} kPa; Boost pressure: {} kPa; Intake manifold temperature: {}°C; Air inlet pressure: {} kPa; Air filter differential pressure: {} kPa; Exhaust gas temperature: {}°C; Coolant filter differential pressure: {} kPa",
            self.particulate_trap_inlet_pressure.unwrap_or(0),
            self.boost_pressure.unwrap_or(0),
            self.intake_manifold_temperature.unwrap_or(0),
            self.air_inlet_pressure.unwrap_or(0),
            self.air_filter_differential_pressure.unwrap_or(0),
            self.exhaust_gas_temperature.unwrap_or(0),
            self.coolant_filter_differential_pressure.unwrap_or(0)
        )
    }
}

//
// Electronic Brake Controller 1
//

pub struct ElectronicBrakeController1Message {
    /// State signal which indicates that ASR engine control has been commanded to be
    /// active. Active means that ASR actually tries to control the engine. This state signal is independent of other control commands to the
    /// engine (e.g., from the transmission) which may have higher priority.
    pub asr_engine_control_active: Option<bool>,
    /// State signal which indicates that ASR brake control is active. Active means that
    /// ASR actually controls wheel brake pressure at one or more wheels of the driven axle(s).
    pub asr_brake_control_active: Option<bool>,
    /// State signal which indicates that the ABS is active. The signal is set active
    /// when wheel brake pressure actually starts to be modulated by ABS and is reset to passive when all wheels are in a stable condition for a
    /// certain time. The signal can also be set active when driven wheels are in high slip (e.g., caused by retarder). Whenever the ABS system
    /// is not fully operational (due to a defect or during off-road ABS operation) , this signal is only valid for that part of the system that is still
    /// working. When ABS is switched off completely, the flag is set to passive regardless of the current wheel slip conditions.
    pub abs_active: Option<bool>,
    /// Switch signal which indicates that the brake pedal is being pressed. The EBS brake switch is
    /// independent of the brake light switch and has no provisions for external connections.
    pub ebs_brake_switch: Option<bool>,
    /// Ratio of brake pedal position to maximum pedal position. Used for electric brake
    /// applications. 0% means no braking. Also when there are two brake pedals on the machine (Left Brake Pedal Position SPN-tba and
    /// Right Brake Pedal Position SPN-tba) the maximum of the two should be transmitted for Brake Pedal Position.
    pub brake_pedal_position: Option<u8>,
    /// Switch signal which indicates the position of the ABS off-road switch.
    pub abs_off_road_switch: Option<bool>,
    /// Switch signal which indicates the position of the ASR off-road switch.
    pub asr_off_road_switch: Option<bool>,
    /// Switch signal which indicates the position of the ASR 'hill holder' switch.
    pub asr_hill_holder_switch: Option<bool>,
    /// Switch signal which indicates the position of the traction control
    /// override switch. The traction control override signal disables the automatic traction control function allowing the wheels to spin.
    pub traction_control_override_switch: Option<bool>,
    /// Switch signal used to disable the accelerator and remote accelerator inputs,
    /// causing the engine to return to idle.
    pub accelerator_interlock_switch: Option<bool>,
    /// Switch signal used to activate the torque limiting feature of the engine. The specific nature
    /// of torque limiting should be verified with the manufacturer.
    pub engine_derate_switch: Option<bool>,
    /// Switch signal which requests that all engine fueling stop.
    pub auxiliary_engine_shutdown_switch: Option<bool>,
    /// Switch signal which indicates that the remote accelerator has been
    /// enabled and controls the engine.
    pub remote_accelerator_enable_switch: Option<bool>,
    /// The position of the operator controlled selector, expressed as a percentage and
    /// determined by the ratio of the current position of the selector to its maximum possible position. Zero percent means no braking torque is
    /// requested by the operator from the engine while 100% means maximum braking.
    pub engine_retarder_selection: Option<u8>,
    /// Signal which indicates whether an ABS system is fully operational or whether its
    /// functionality is reduced by a defect or by an intended action (e.g., by activation of an ABS-off-road switch or during special diagnostic
    /// procedures). There are cases where the signal is necessary to fulfill legal regulations for special applications (e.g., switching off
    /// integrated retarders).
    pub abs_fully_operational: Option<bool>,
    /// Status signal which indicates fuel leakage in the fuel rail of the engine. The location can be either
    /// before or after the fuel pump.
    pub ebs_red_warning_signal: Option<bool>,
    /// This parameter commands the ABS/EBS amber/yellow optical warning signal.
    pub abs_ebs_amber_warning_signal: Option<bool>,
    /// This parameter commands the ATC/ASR driver information signal, for example a dash lamp.
    pub atc_asr_information_signal: Option<bool>,
    /// The source address of the SAE J1939 device currently controlling the brake system. Its value may be the source address of the ECU
    /// transmitting the message (which means that no external SAE J1939 message is providing the active command) or the source address of
    /// the SAE J1939 ECU that is currently providing the active command in a TSC1 (see PGN 0) or similar message. Note that if this parameter
    /// value is the same as the source address of the device transmitting it, the control may be due to a message on a non-SAE J1939 data link
    /// such as SAE J1922 or a proprietary link.
    pub source_address: Option<u8>,
    /// State signal which indicates that ABS in the trailer is actively controlling the brakes. A
    /// message is sent to the tractor from the trailer (i.e. by PLC). The receiving device in the tractor transfers this information to the J1939
    /// network. At the beginning of power on the message is sent by the trailer to indicate if this status information is supported. Timeout of
    /// the trailer ABS active can be done by monitoring of the Trailer warning light information.
    pub trailer_abs_status: Option<bool>,
    /// This parameter commands the tractor-mounted trailer ABS optical warning signal.
    pub tractor_mounted_trailer_abs_warning_signal: Option<bool>,
}

impl ElectronicBrakeController1Message {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "ElectronicBrakeController1Message::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            asr_engine_control_active: slots::bool_from_value(pdu[0]),
            asr_brake_control_active: slots::bool_from_value(pdu[0] >> 2),
            abs_active: slots::bool_from_value(pdu[0] >> 4),
            ebs_brake_switch: slots::bool_from_value(pdu[0] >> 6),
            brake_pedal_position: slots::position_level::dec(pdu[1]),
            abs_off_road_switch: slots::bool_from_value(pdu[2]),
            asr_off_road_switch: slots::bool_from_value(pdu[2] >> 2),
            asr_hill_holder_switch: slots::bool_from_value(pdu[2] >> 4),
            traction_control_override_switch: slots::bool_from_value(pdu[2] >> 6),
            accelerator_interlock_switch: slots::bool_from_value(pdu[3]),
            engine_derate_switch: slots::bool_from_value(pdu[3] >> 2),
            auxiliary_engine_shutdown_switch: slots::bool_from_value(pdu[3] >> 4),
            remote_accelerator_enable_switch: slots::bool_from_value(pdu[3] >> 6),
            engine_retarder_selection: slots::position_level::dec(pdu[4]),
            abs_fully_operational: slots::bool_from_value(pdu[5]),
            ebs_red_warning_signal: slots::bool_from_value(pdu[5] >> 2),
            abs_ebs_amber_warning_signal: slots::bool_from_value(pdu[5] >> 4),
            atc_asr_information_signal: slots::bool_from_value(pdu[5] >> 6),
            source_address: slots::source_address::dec(pdu[6]),
            trailer_abs_status: slots::bool_from_value(pdu[7] >> 4),
            tractor_mounted_trailer_abs_warning_signal: slots::bool_from_value(pdu[7] >> 6),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::bool_to_value(self.asr_engine_control_active)
                | slots::bool_to_value(self.asr_brake_control_active) << 2
                | slots::bool_to_value(self.abs_active) << 4
                | slots::bool_to_value(self.ebs_brake_switch) << 6,
            slots::position_level::enc(self.brake_pedal_position),
            slots::bool_to_value(self.abs_off_road_switch)
                | slots::bool_to_value(self.asr_off_road_switch) << 2
                | slots::bool_to_value(self.asr_hill_holder_switch) << 4
                | slots::bool_to_value(self.traction_control_override_switch) << 6,
            slots::bool_to_value(self.accelerator_interlock_switch)
                | slots::bool_to_value(self.engine_derate_switch) << 2
                | slots::bool_to_value(self.auxiliary_engine_shutdown_switch) << 4
                | slots::bool_to_value(self.remote_accelerator_enable_switch) << 6,
            slots::position_level::enc(self.engine_retarder_selection),
            slots::bool_to_value(self.abs_fully_operational)
                | slots::bool_to_value(self.ebs_red_warning_signal) << 2
                | slots::bool_to_value(self.abs_ebs_amber_warning_signal) << 4
                | slots::bool_to_value(self.atc_asr_information_signal) << 6,
            slots::source_address::enc(self.source_address),
            slots::bool_to_value(self.trailer_abs_status) << 4
                | slots::bool_to_value(self.tractor_mounted_trailer_abs_warning_signal) << 6,
        ]
    }
}

impl core::fmt::Display for ElectronicBrakeController1Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ASR engine control active: {:?}; ASR brake control active: {:?}; ABS active: {:?}; EBS brake switch: {:?}; Brake pedal position: {}%; ABS off-road switch: {:?}; ASR off-road switch: {:?}; ASR hill holder switch: {:?}; Traction control override switch: {:?}; Accelerator interlock switch: {:?}; Engine derate switch: {:?}; Auxiliary engine shutdown switch: {:?}; Remote accelerator enable switch: {:?}; Engine retarder selection: {}%; ABS fully operational: {:?}; EBS red warning signal: {:?}; ABS/EBS amber warning signal: {:?}; ATC/ASR information signal: {:?}; Source address: {:?}; Trailer ABS status: {:?}; Tractor-mounted trailer ABS warning signal: {:?}",
            self.asr_engine_control_active,
            self.asr_brake_control_active,
            self.abs_active,
            self.ebs_brake_switch,
            self.brake_pedal_position.unwrap_or(0),
            self.abs_off_road_switch,
            self.asr_off_road_switch,
            self.asr_hill_holder_switch,
            self.traction_control_override_switch,
            self.accelerator_interlock_switch,
            self.engine_derate_switch,
            self.auxiliary_engine_shutdown_switch,
            self.remote_accelerator_enable_switch,
            self.engine_retarder_selection.unwrap_or(0),
            self.abs_fully_operational,
            self.ebs_red_warning_signal,
            self.abs_ebs_amber_warning_signal,
            self.atc_asr_information_signal,
            self.source_address,
            self.trailer_abs_status,
            self.tractor_mounted_trailer_abs_warning_signal
        )
    }
}

//
// TANK Information 1
//

pub struct TankInformation1Message {
    /// A special catalyst uses chemical substance to reach legal requirement for NOX emissions.
    /// This parameter indicates the level within that catalyst tank. 0 % = Empty 100% = Full.
    pub catalyst_tank_level: Option<u8>,
}

impl TankInformation1Message {
    /// # Panics
    /// Panics if `pdu` is empty.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(!pdu.is_empty(), "TankInformation1Message::from_pdu requires at least 1 byte, got 0");
        Self {
            catalyst_tank_level: slots::position_level::dec(pdu[0]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::position_level::enc(self.catalyst_tank_level),
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
            PDU_NOT_AVAILABLE,
        ]
    }
}

impl core::fmt::Display for TankInformation1Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Catalyst tank level: {}%",
            self.catalyst_tank_level.unwrap_or(0)
        )
    }
}

//
// Vehicle Electrical Power
//

pub struct VehicleElectricalPowerMessage {
    /// Net flow of electrical current into/out of the battery or batteries.
    pub net_battery_current: Option<i8>,
    /// Measure of electrical current flow from the alternator. Alternator Current (High
    /// Range/Resolution) parameter SPN 1795 has a higher range and resolution of the same parameter.
    pub alternator_current: Option<u8>,
    /// Electrical potential measured at the alternator output.
    pub alternator_potential: Option<u16>,
    /// Measured electrical potential of the battery.
    pub electrical_potential: Option<u16>,
    /// Electrical potential measured at the input of the electronic control
    /// unit supplied through a switching device.
    pub battery_potential: Option<u16>,
}

impl VehicleElectricalPowerMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "VehicleElectricalPowerMessage::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            net_battery_current: slots::electrical_current::dec(pdu[0]),
            alternator_current: slots::electrical_current2::dec(pdu[1]),
            alternator_potential: slots::electrical_voltage::dec([pdu[2], pdu[3]]),
            electrical_potential: slots::electrical_voltage::dec([pdu[4], pdu[5]]),
            battery_potential: slots::electrical_voltage::dec([pdu[6], pdu[7]]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::electrical_current::enc(self.net_battery_current),
            slots::electrical_current2::enc(self.alternator_current),
            slots::electrical_voltage::enc(self.alternator_potential)[0],
            slots::electrical_voltage::enc(self.alternator_potential)[1],
            slots::electrical_voltage::enc(self.electrical_potential)[0],
            slots::electrical_voltage::enc(self.electrical_potential)[1],
            slots::electrical_voltage::enc(self.battery_potential)[0],
            slots::electrical_voltage::enc(self.battery_potential)[1],
        ]
    }
}

impl core::fmt::Display for VehicleElectricalPowerMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Net battery current: {} A; Alternator current: {} A; Alternator potential: {} V; Electrical potential: {} V; Battery potential: {} V",
            self.net_battery_current.unwrap_or(0),
            self.alternator_current.unwrap_or(0),
            self.alternator_potential.unwrap_or(0),
            self.electrical_potential.unwrap_or(0),
            self.battery_potential.unwrap_or(0)
        )
    }
}

//
// Engine Fluid Level/Pressure 2
//

pub struct EngineFluidLevelPressure2Message {
    /// The gage pressure of the engine oil in the hydraulic accumulator that powers an
    /// intensifier used for fuel injection.
    pub injection_control_pressure: Option<u16>,
    /// The gage pressure of fuel in the primary, or first, metering rail as
    /// delivered from the supply pump to the injector metering inlet.
    pub injector_metering_rail1_pressure: Option<u16>,
    /// The gage pressure of fuel in the timing rail delivered from the supply pump
    /// to the injector timing inlet.
    pub injector_timing_rail1_pressure: Option<u16>,
    /// The gage pressure of fuel in the metering rail #2 as delivered from the
    /// supply pump to the injector metering inlet.
    pub injector_metering_rail2_pressure: Option<u16>,
}

impl EngineFluidLevelPressure2Message {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(pdu.len() >= 8, "EngineFluidLevelPressure2Message::from_pdu requires at least 8 bytes, got {}", pdu.len());
        Self {
            injection_control_pressure: slots::pressure5::dec([pdu[0], pdu[1]]),
            injector_metering_rail1_pressure: slots::pressure5::dec([pdu[2], pdu[3]]),
            injector_timing_rail1_pressure: slots::pressure5::dec([pdu[4], pdu[5]]),
            injector_metering_rail2_pressure: slots::pressure5::dec([pdu[6], pdu[7]]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::pressure5::enc(self.injection_control_pressure)[0],
            slots::pressure5::enc(self.injection_control_pressure)[1],
            slots::pressure5::enc(self.injector_metering_rail1_pressure)[0],
            slots::pressure5::enc(self.injector_metering_rail1_pressure)[1],
            slots::pressure5::enc(self.injector_timing_rail1_pressure)[0],
            slots::pressure5::enc(self.injector_timing_rail1_pressure)[1],
            slots::pressure5::enc(self.injector_metering_rail2_pressure)[0],
            slots::pressure5::enc(self.injector_metering_rail2_pressure)[1],
        ]
    }
}

impl core::fmt::Display for EngineFluidLevelPressure2Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Injection control pressure: {} MPa; Injector metering rail 1 pressure: {} MPa; Injector timing rail 1 pressure: {} MPa; Injector metering rail 2 pressure: {} MPa",
            self.injection_control_pressure.unwrap_or(0),
            self.injector_metering_rail1_pressure.unwrap_or(0),
            self.injector_timing_rail1_pressure.unwrap_or(0),
            self.injector_metering_rail2_pressure.unwrap_or(0)
        )
    }
}

//
// Reset (PGN 56832)
//

pub struct ResetMessage {
    /// Command signal used to reset the PGNs and parameters as defined in Table `SPN988_A`.
    pub trip_group_1: Option<bool>,
    /// Command signal used to reset proprietary parameters associated with a trip but not
    /// defined within this document.
    pub trip_group_2_proprietary: Option<bool>,
    /// Identification of component needing service.
    pub service_component_identification: Option<u8>,
    /// Command signal used to reset the engine rebuild hours.
    pub engine_build_hours_reset: Option<bool>,
    /// Used to reset the straight ahead position for a steering sensor in the steering
    /// column or a steering controller's straight ahead position on any steerable axle.
    pub steering_straight_ahead_position_reset: Option<bool>,
    /// Command signal used to reset the ignition controller average, maximum, and minimum
    /// level tracking of the spark plug secondary voltages and to reset the learned misfire rate.
    pub engine_spark_plug_secondary_voltage_tracking_reset: Option<bool>,
    /// Used to reset the maintenance hour counter for an engine ignition control module.
    pub engine_ignition_control_maintenance_hours_reset: Option<bool>,
    /// Used to reset the bin lift count as reported in PGN 64594.
    pub bin_lift_count_reset: Option<bool>,
    /// Command signal used to initiate change in the tire configuration of the vehicle system.
    pub tire_configuration_information: Option<bool>,
    /// Command signal used to initiate change in the tire sensor identification information.
    pub tire_sensor_information: Option<bool>,
}

impl ResetMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(
            pdu.len() >= 8,
            "ResetMessage::from_pdu requires at least 8 bytes, got {}",
            pdu.len()
        );
        Self {
            trip_group_1: slots::bool_from_value(pdu[0]),
            trip_group_2_proprietary: slots::bool_from_value(pdu[0] >> 2),
            service_component_identification: slots::id::dec(pdu[1]),
            engine_build_hours_reset: slots::bool_from_value(pdu[2]),
            steering_straight_ahead_position_reset: slots::bool_from_value(pdu[2] >> 2),
            engine_spark_plug_secondary_voltage_tracking_reset: slots::bool_from_value(pdu[2] >> 4),
            engine_ignition_control_maintenance_hours_reset: slots::bool_from_value(pdu[2] >> 6),
            bin_lift_count_reset: slots::bool_from_value(pdu[3]),
            tire_configuration_information: slots::bool_from_value(pdu[3] >> 2),
            tire_sensor_information: slots::bool_from_value(pdu[3] >> 4),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        [
            slots::bool_to_value(self.trip_group_1)
                | (slots::bool_to_value(self.trip_group_2_proprietary) << 2)
                | 0xF0,
            slots::id::enc(self.service_component_identification),
            slots::bool_to_value(self.engine_build_hours_reset)
                | (slots::bool_to_value(self.steering_straight_ahead_position_reset) << 2)
                | (slots::bool_to_value(self.engine_spark_plug_secondary_voltage_tracking_reset) << 4)
                | (slots::bool_to_value(self.engine_ignition_control_maintenance_hours_reset) << 6),
            slots::bool_to_value(self.bin_lift_count_reset)
                | (slots::bool_to_value(self.tire_configuration_information) << 2)
                | (slots::bool_to_value(self.tire_sensor_information) << 4)
                | 0xC0,
            0xFF,
            0xFF,
            0xFF,
            0xFF,
        ]
    }
}

impl core::fmt::Display for ResetMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Trip Group 1: {:?}; Trip Group 2 Proprietary: {:?}; Service Component ID: {:?}; Engine Build Hours Reset: {:?}; Steering Position Reset: {:?}; Spark Plug Reset: {:?}; Maintenance Hours Reset: {:?}; Bin Lift Count Reset: {:?}; Tire Config Info: {:?}; Tire Sensor Info: {:?}",
            self.trip_group_1,
            self.trip_group_2_proprietary,
            self.service_component_identification,
            self.engine_build_hours_reset,
            self.steering_straight_ahead_position_reset,
            self.engine_spark_plug_secondary_voltage_tracking_reset,
            self.engine_ignition_control_maintenance_hours_reset,
            self.bin_lift_count_reset,
            self.tire_configuration_information,
            self.tire_sensor_information
        )
    }
}

//
// Acknowledgment (PGN 59392)
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcknowledgmentType {
    /// Positive Acknowledgment
    Positive,
    /// Negative Acknowledgment
    Negative,
    /// Access Denied
    AccessDenied,
    /// Cannot respond (e.g. busy)
    Busy,
}

impl AcknowledgmentType {
    #[must_use]
    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Positive),
            1 => Some(Self::Negative),
            2 => Some(Self::AccessDenied),
            3 => Some(Self::Busy),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_value(self) -> u8 {
        match self {
            Self::Positive => 0,
            Self::Negative => 1,
            Self::AccessDenied => 2,
            Self::Busy => 3,
        }
    }
}

pub struct AcknowledgmentMessage {
    /// Control byte indicating the type of acknowledgment.
    pub control_byte: Option<AcknowledgmentType>,
    /// Group function value related to the acknowledgment.
    pub group_function_value: u8,
    /// Parameter Group Number being acknowledged.
    pub pgn: crate::pgn::PGN,
}

impl AcknowledgmentMessage {
    /// # Panics
    /// Panics if `pdu` has fewer than 8 bytes.
    #[must_use]
    pub fn from_pdu(pdu: &[u8]) -> Self {
        assert!(
            pdu.len() >= 8,
            "AcknowledgmentMessage::from_pdu requires at least 8 bytes, got {}",
            pdu.len()
        );
        Self {
            control_byte: AcknowledgmentType::from_value(pdu[0]),
            group_function_value: pdu[1],
            pgn: crate::pgn::PGN::from_le_bytes([pdu[5], pdu[6], pdu[7]]),
        }
    }

    #[must_use]
    pub fn to_pdu(&self) -> [u8; 8] {
        let pgn_bytes = self.pgn.to_le_bytes();
        [
            self.control_byte.map_or(0xFF, AcknowledgmentType::to_value),
            self.group_function_value,
            0xFF, // Reserved
            0xFF, // Reserved
            0xFF, // Reserved
            pgn_bytes[0],
            pgn_bytes[1],
            pgn_bytes[2],
        ]
    }
}

impl core::fmt::Display for AcknowledgmentMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Acknowledgment: {:?}; Group Function: {}; PGN: {:?}",
            self.control_byte, self.group_function_value, self.pgn
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn electronic_engine_controller_1_message_1() {
        let engine_message = ElectronicEngineController1Message::from_pdu(&[
            0xF0, 0xEA, 0x7D, 0x00, 0x00, 0x00, 0xF0, 0xFF,
        ]);
        assert_eq!(
            engine_message.engine_torque_mode,
            Some(EngineTorqueMode::NoRequest)
        );
        assert_eq!(engine_message.driver_demand, Some(109));
        assert_eq!(engine_message.actual_engine, Some(0));
        assert_eq!(engine_message.rpm, Some(0));
        assert_eq!(engine_message.source_addr, Some(0));
        assert_eq!(
            engine_message.starter_mode,
            Some(EngineStarterMode::StartNotRequested)
        );
    }

    #[test]
    fn electronic_engine_controller_1_message_2() {
        let engine_message = ElectronicEngineController1Message::from_pdu(&[
            0xF3, 0x91, 0x91, 0xAA, 0x18, 0x00, 0xF3, 0xFF,
        ]);
        assert_eq!(
            engine_message.engine_torque_mode,
            Some(EngineTorqueMode::PTOGovernor)
        );
        assert_eq!(engine_message.driver_demand, Some(20));
        assert_eq!(engine_message.actual_engine, Some(20));
        assert_eq!(engine_message.rpm, Some(789));
        assert_eq!(engine_message.source_addr, Some(0));
        assert_eq!(
            engine_message.starter_mode,
            Some(EngineStarterMode::StartFinished)
        );
    }

    #[test]
    fn electronic_engine_controller_1_message_3() {
        let engine_message = ElectronicEngineController1Message::from_pdu(&[
            0xFF, 0x91, 0x91, 0xAA, 0x18, 0x00, 0xFF, 0xFF,
        ]);
        assert_eq!(engine_message.engine_torque_mode, None);
        assert_eq!(engine_message.driver_demand, Some(20));
        assert_eq!(engine_message.actual_engine, Some(20));
        assert_eq!(engine_message.rpm, Some(789));
        assert_eq!(engine_message.source_addr, Some(0));
        assert_eq!(engine_message.starter_mode, None);
    }

    #[test]
    fn electronic_engine_controller_1_message_4() {
        let engine_message = ElectronicEngineController1Message::from_pdu(&[
            0xFF, 0x00, 0x7D, 0x00, 0x00, 0x32, 0xFF, 0xFF,
        ]);
        assert_eq!(engine_message.engine_torque_mode, None);
        assert_eq!(engine_message.driver_demand, Some(0));
        assert_eq!(engine_message.actual_engine, Some(0));
        assert_eq!(engine_message.rpm, Some(0));
        assert_eq!(engine_message.source_addr, Some(0x32));
        assert_eq!(engine_message.starter_mode, None);
    }

    #[test]
    fn electronic_engine_controller_1_message_5() {
        let engine_message = ElectronicEngineController1Message::from_pdu(&[
            0xFF, 0xAC, 0x7D, 0x00, 0x00, 0x32, 0x00, 0x00,
        ]);
        assert_eq!(engine_message.engine_torque_mode, None);
        assert_eq!(engine_message.driver_demand, Some(47));
        assert_eq!(engine_message.actual_engine, Some(0));
        assert_eq!(engine_message.rpm, Some(0));
        assert_eq!(engine_message.source_addr, Some(0x32));
        assert_eq!(
            engine_message.starter_mode,
            Some(EngineStarterMode::StartNotRequested)
        );
    }

    #[test]
    fn electronic_engine_controller_1_message_6() {
        let engine_message_encoded = ElectronicEngineController1Message {
            engine_torque_mode: Some(EngineTorqueMode::HighSpeedGovernor),
            driver_demand: Some(93),
            actual_engine: Some(4),
            rpm: Some(2156),
            source_addr: Some(21),
            starter_mode: Some(EngineStarterMode::StarterInhibitedOverHeat),
        }
        .to_pdu();
        let engine_message_decoded =
            ElectronicEngineController1Message::from_pdu(&engine_message_encoded);

        assert_eq!(
            engine_message_decoded.engine_torque_mode,
            Some(EngineTorqueMode::HighSpeedGovernor)
        );
        assert_eq!(engine_message_decoded.driver_demand, Some(93));
        assert_eq!(engine_message_decoded.actual_engine, Some(4));
        assert_eq!(engine_message_decoded.rpm, Some(2156));
        assert_eq!(engine_message_decoded.source_addr, Some(21));
        assert_eq!(
            engine_message_decoded.starter_mode,
            Some(EngineStarterMode::StarterInhibitedOverHeat)
        );
    }

    #[test]
    fn electronic_engine_controller_3_message_1() {
        let engine_message = ElectronicEngineController3Message::from_pdu(&[
            0xFF, 0x00, 0x00, 0xC0, 0x5D, 0x40, 0x00, 0x00,
        ]);

        assert_eq!(engine_message.nominal_friction_percent_torque, None);
        assert_eq!(engine_message.engines_desired_operating_speed, Some(0));
        assert_eq!(
            engine_message.engines_desired_operating_speed_asymmetry_adjustment,
            Some(192)
        );
    }

    #[test]
    fn electronic_engine_controller_3_message_2() {
        let engine_message_encoded = ElectronicEngineController3Message {
            nominal_friction_percent_torque: Some(50),
            engines_desired_operating_speed: Some(3632),
            engines_desired_operating_speed_asymmetry_adjustment: Some(23),
        }
        .to_pdu();
        let engine_message_decoded =
            ElectronicEngineController3Message::from_pdu(&engine_message_encoded);

        assert_eq!(
            engine_message_decoded.nominal_friction_percent_torque,
            Some(50)
        );
        assert_eq!(
            engine_message_decoded.engines_desired_operating_speed,
            Some(3632)
        );
        assert_eq!(
            engine_message_decoded.engines_desired_operating_speed_asymmetry_adjustment,
            Some(23)
        );
    }

    #[test]
    fn torque_speed_control_1_message_1() {
        let torque_speed =
            TorqueSpeedControl1Message::from_pdu(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        assert_eq!(
            torque_speed.override_control_mode,
            OverrideControlMode::OverrideDisabled
        );
        assert_eq!(
            torque_speed.speed_control_condition,
            RequestedSpeedControlCondition::TransientOptimizedDriveLineDisengaged
        );
        assert_eq!(
            torque_speed.control_mode_priority,
            OverrideControlModePriority::HighestPriority
        );
        assert_eq!(torque_speed.speed, Some(0));
        assert_eq!(torque_speed.torque, Some(0));
    }

    #[test]
    fn torque_speed_control_1_message_2() {
        let torque_speed_encoded = TorqueSpeedControl1Message {
            override_control_mode: OverrideControlMode::SpeedControl,
            speed_control_condition:
                RequestedSpeedControlCondition::StabilityOptimizedDriveLineEngaged1,
            control_mode_priority: OverrideControlModePriority::MediumPriority,
            speed: Some(1234),
            torque: Some(56),
        }
        .to_pdu();
        let torque_speed_decoded = TorqueSpeedControl1Message::from_pdu(&torque_speed_encoded);

        assert_eq!(
            torque_speed_decoded.override_control_mode,
            OverrideControlMode::SpeedControl
        );
        assert_eq!(
            torque_speed_decoded.speed_control_condition,
            RequestedSpeedControlCondition::StabilityOptimizedDriveLineEngaged1
        );
        assert_eq!(
            torque_speed_decoded.control_mode_priority,
            OverrideControlModePriority::MediumPriority
        );
        assert_eq!(torque_speed_decoded.speed, Some(1234));
        assert_eq!(torque_speed_decoded.torque, Some(56));
    }

    #[test]
    fn torque_speed_control_1_message_3() {
        let torque_speed_encoded = TorqueSpeedControl1Message {
            override_control_mode: OverrideControlMode::SpeedTorqueLimitControl,
            speed_control_condition:
                RequestedSpeedControlCondition::StabilityOptimizedDriveLineEngaged1,
            control_mode_priority: OverrideControlModePriority::MediumPriority,
            speed: None,
            torque: None,
        }
        .to_pdu();
        let torque_speed_decoded = TorqueSpeedControl1Message::from_pdu(&torque_speed_encoded);

        assert_eq!(
            torque_speed_decoded.override_control_mode,
            OverrideControlMode::SpeedTorqueLimitControl
        );
        assert_eq!(
            torque_speed_decoded.speed_control_condition,
            RequestedSpeedControlCondition::StabilityOptimizedDriveLineEngaged1
        );
        assert_eq!(
            torque_speed_decoded.control_mode_priority,
            OverrideControlModePriority::MediumPriority
        );
        assert_eq!(torque_speed_decoded.speed, None);
        assert_eq!(torque_speed_decoded.torque, None);
    }

    #[test]
    fn ambient_conditions_message_1() {
        let engine_temperature =
            AmbientConditionsMessage::from_pdu(&[0xC0, 0xFF, 0xFF, 0xFF, 0xFF, 0x35, 0xFF, 0xFF]);

        assert_eq!(engine_temperature.barometric_pressure, Some(96));
        assert_eq!(engine_temperature.cab_interior_temperature, None);
        assert_eq!(engine_temperature.ambient_air_temperature, None);
        assert_eq!(engine_temperature.air_inlet_temperature, Some(13));
        assert_eq!(engine_temperature.road_surface_temperature, None);
    }

    #[test]
    fn engine_fluid_level_pressure_1_message_1() {
        let engine_fluid_level_pressure = EngineFluidLevelPressure1Message::from_pdu(&[
            0x1A, 0xFF, 0xFF, 0x01, 0xFF, 0xFF, 0x00, 0x00,
        ]);

        assert_eq!(
            engine_fluid_level_pressure.fuel_delivery_pressure,
            Some(104)
        );
        assert_eq!(
            engine_fluid_level_pressure.extended_crankcase_blow_by_pressure,
            None
        );
        assert_eq!(engine_fluid_level_pressure.engine_oil_level, None);
        assert_eq!(engine_fluid_level_pressure.engine_oil_pressure, Some(4));
        assert_eq!(engine_fluid_level_pressure.crankcase_pressure, None);
        assert_eq!(engine_fluid_level_pressure.coolant_pressure, Some(0));
        assert_eq!(engine_fluid_level_pressure.coolant_level, Some(0));
    }

    #[test]
    fn fuel_consumption_message_1() {
        let fuel_consumption =
            FuelConsumptionMessage::from_pdu(&[0xFA, 0xD8, 0x02, 0x00, 0xFA, 0xD8, 0x02, 0x00]);

        assert_eq!(fuel_consumption.trip_fuel, Some(93309));
        assert_eq!(fuel_consumption.total_fuel_used, Some(93309));
    }

    #[test]
    fn fuel_consumption_message_2() {
        let fuel_consumption_encoded = FuelConsumptionMessage {
            trip_fuel: Some(1234),
            total_fuel_used: Some(56),
        }
        .to_pdu();
        let fuel_consumption_decoded = FuelConsumptionMessage::from_pdu(&fuel_consumption_encoded);

        assert_eq!(fuel_consumption_decoded.trip_fuel, Some(1234));
        assert_eq!(fuel_consumption_decoded.total_fuel_used, Some(56));
    }

    #[test]
    fn fan_drive_message_1() {
        let fan_drive_encoded = FanDriveMessage {
            estimated_percent_fan_speed: Some(50),
            fan_drive_state: Some(FanDriveState::ExcessiveHydraulicOilTemperature),
            fan_speed: Some(1000),
        }
        .to_pdu();

        let fan_drive_decoded = FanDriveMessage::from_pdu(&fan_drive_encoded);

        assert_eq!(fan_drive_decoded.estimated_percent_fan_speed, Some(50));
        assert_eq!(
            fan_drive_decoded.fan_drive_state,
            Some(FanDriveState::ExcessiveHydraulicOilTemperature)
        );
        assert_eq!(fan_drive_decoded.fan_speed, Some(1000));
    }

    #[test]
    fn fan_drive_message_2() {
        let fan_drive_encoded = FanDriveMessage {
            estimated_percent_fan_speed: None,
            fan_drive_state: None,
            fan_speed: None,
        }
        .to_pdu();

        let fan_drive_decoded = FanDriveMessage::from_pdu(&fan_drive_encoded);

        assert_eq!(fan_drive_decoded.estimated_percent_fan_speed, None);
        assert_eq!(fan_drive_decoded.fan_drive_state, None);
        assert_eq!(fan_drive_decoded.fan_speed, None);
    }

    #[test]
    fn shutdown_message_1() {
        let shutdown = ShutdownMessage::from_pdu(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        assert_eq!(shutdown.idle_shutdown_has_shutdown_engine, Some(false));
        assert_eq!(shutdown.idle_shutdown_driver_alert_mode, Some(false));
        assert_eq!(shutdown.idle_shutdown_timer_override, Some(false));
        assert_eq!(shutdown.idle_shutdown_timer_state, Some(false));
        assert_eq!(shutdown.idle_shutdown_timer_function, Some(false));
        assert_eq!(shutdown.ac_high_pressure_fan_switch, Some(false));
        assert_eq!(shutdown.refrigerant_low_pressure_switch, Some(false));
        assert_eq!(shutdown.refrigerant_high_pressure_switch, Some(false));
        assert_eq!(shutdown.wait_to_start_lamp, Some(false));
        assert_eq!(
            shutdown.engine_protection_system_has_shutdown_engine,
            Some(false)
        );
        assert_eq!(
            shutdown.engine_protection_system_approaching_shutdown,
            Some(false)
        );
        assert_eq!(
            shutdown.engine_protection_system_timer_override,
            Some(false)
        );
        assert_eq!(shutdown.engine_protection_system_timer_state, Some(false));
        assert_eq!(shutdown.engine_protection_system_configuration, Some(false));
    }

    #[test]
    fn engine_temperature_1_message_1() {
        let engine_temperature =
            EngineTemperature1Message::from_pdu(&[0x42, 0x3B, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

        assert_eq!(engine_temperature.engine_coolant_temperature, Some(26));
        assert_eq!(engine_temperature.fuel_temperature, Some(19));
        assert_eq!(engine_temperature.engine_oil_temperature, None);
        assert_eq!(engine_temperature.turbo_oil_temperature, None);
        assert_eq!(engine_temperature.engine_intercooler_temperature, None);
        assert_eq!(
            engine_temperature.engine_intercooler_thermostat_opening,
            None
        );
    }

    #[test]
    fn inlet_exhaust_conditions_1_message_1() {
        let inlet_exhaust_conditions = InletExhaustConditions1Message::from_pdu(&[
            0xD4, 0x30, 0x3E, 0x32, 0x41, 0x0B, 0x0A, 0x00,
        ]);

        assert_eq!(
            inlet_exhaust_conditions.particulate_trap_inlet_pressure,
            None
        );
        assert_eq!(inlet_exhaust_conditions.boost_pressure, Some(96));
        assert_eq!(
            inlet_exhaust_conditions.intake_manifold_temperature,
            Some(22)
        );
        assert_eq!(inlet_exhaust_conditions.air_inlet_pressure, Some(100));
        assert_eq!(
            inlet_exhaust_conditions.air_filter_differential_pressure,
            Some(3)
        );
        assert_eq!(inlet_exhaust_conditions.exhaust_gas_temperature, Some(-192));
        assert_eq!(
            inlet_exhaust_conditions.coolant_filter_differential_pressure,
            None
        );
    }

    #[test]
    fn electronic_brake_controller_1_message_1() {
        let brake_message_encoded = ElectronicBrakeController1Message {
            asr_engine_control_active: Some(false),
            asr_brake_control_active: Some(true),
            abs_active: Some(false),
            ebs_brake_switch: Some(true),
            brake_pedal_position: Some(2),
            abs_off_road_switch: Some(false),
            asr_off_road_switch: Some(false),
            asr_hill_holder_switch: Some(true),
            traction_control_override_switch: Some(true),
            accelerator_interlock_switch: Some(true),
            engine_derate_switch: Some(false),
            auxiliary_engine_shutdown_switch: Some(true),
            remote_accelerator_enable_switch: Some(false),
            engine_retarder_selection: Some(64),
            abs_fully_operational: Some(false),
            ebs_red_warning_signal: Some(false),
            abs_ebs_amber_warning_signal: Some(true),
            atc_asr_information_signal: Some(false),
            source_address: Some(0),
            trailer_abs_status: Some(false),
            tractor_mounted_trailer_abs_warning_signal: Some(true),
        }
        .to_pdu();

        let brake_message_decoded =
            ElectronicBrakeController1Message::from_pdu(&brake_message_encoded);

        assert_eq!(brake_message_decoded.asr_engine_control_active, Some(false));
        assert_eq!(brake_message_decoded.asr_brake_control_active, Some(true));
        assert_eq!(brake_message_decoded.abs_active, Some(false));
        assert_eq!(brake_message_decoded.ebs_brake_switch, Some(true));
        assert_eq!(brake_message_decoded.brake_pedal_position, Some(2));
        assert_eq!(brake_message_decoded.abs_off_road_switch, Some(false));
        assert_eq!(brake_message_decoded.asr_off_road_switch, Some(false));
        assert_eq!(brake_message_decoded.asr_hill_holder_switch, Some(true));
        assert_eq!(
            brake_message_decoded.traction_control_override_switch,
            Some(true)
        );
        assert_eq!(
            brake_message_decoded.accelerator_interlock_switch,
            Some(true)
        );
        assert_eq!(brake_message_decoded.engine_derate_switch, Some(false));
        assert_eq!(
            brake_message_decoded.auxiliary_engine_shutdown_switch,
            Some(true)
        );
        assert_eq!(
            brake_message_decoded.remote_accelerator_enable_switch,
            Some(false)
        );
        assert_eq!(brake_message_decoded.engine_retarder_selection, Some(64));
        assert_eq!(brake_message_decoded.abs_fully_operational, Some(false));
        assert_eq!(brake_message_decoded.ebs_red_warning_signal, Some(false));
        assert_eq!(
            brake_message_decoded.abs_ebs_amber_warning_signal,
            Some(true)
        );
        assert_eq!(
            brake_message_decoded.atc_asr_information_signal,
            Some(false)
        );
        assert_eq!(brake_message_decoded.source_address, Some(0));
        assert_eq!(brake_message_decoded.trailer_abs_status, Some(false));
        assert_eq!(
            brake_message_decoded.tractor_mounted_trailer_abs_warning_signal,
            Some(true)
        );
    }

    #[test]
    fn electronic_brake_controller_1_message_2() {
        let brake_message_encoded = ElectronicBrakeController1Message {
            asr_engine_control_active: None,
            asr_brake_control_active: None,
            abs_active: None,
            ebs_brake_switch: None,
            brake_pedal_position: None,
            abs_off_road_switch: None,
            asr_off_road_switch: None,
            asr_hill_holder_switch: None,
            traction_control_override_switch: None,
            accelerator_interlock_switch: None,
            engine_derate_switch: None,
            auxiliary_engine_shutdown_switch: Some(true),
            remote_accelerator_enable_switch: None,
            engine_retarder_selection: None,
            abs_fully_operational: None,
            ebs_red_warning_signal: None,
            abs_ebs_amber_warning_signal: None,
            atc_asr_information_signal: None,
            source_address: None,
            trailer_abs_status: None,
            tractor_mounted_trailer_abs_warning_signal: None,
        }
        .to_pdu();

        let brake_message_decoded =
            ElectronicBrakeController1Message::from_pdu(&brake_message_encoded);

        assert_eq!(brake_message_decoded.asr_engine_control_active, None);
        assert_eq!(brake_message_decoded.asr_brake_control_active, None);
        assert_eq!(brake_message_decoded.abs_active, None);
        assert_eq!(brake_message_decoded.ebs_brake_switch, None);
        assert_eq!(brake_message_decoded.brake_pedal_position, None);
        assert_eq!(brake_message_decoded.abs_off_road_switch, None);
        assert_eq!(brake_message_decoded.asr_off_road_switch, None);
        assert_eq!(brake_message_decoded.asr_hill_holder_switch, None);
        assert_eq!(brake_message_decoded.traction_control_override_switch, None);
        assert_eq!(brake_message_decoded.accelerator_interlock_switch, None);
        assert_eq!(brake_message_decoded.engine_derate_switch, None);
        assert_eq!(
            brake_message_decoded.auxiliary_engine_shutdown_switch,
            Some(true)
        );
        assert_eq!(brake_message_decoded.remote_accelerator_enable_switch, None);
        assert_eq!(brake_message_decoded.engine_retarder_selection, None);
        assert_eq!(brake_message_decoded.abs_fully_operational, None);
        assert_eq!(brake_message_decoded.ebs_red_warning_signal, None);
        assert_eq!(brake_message_decoded.abs_ebs_amber_warning_signal, None);
        assert_eq!(brake_message_decoded.atc_asr_information_signal, None);
        assert_eq!(brake_message_decoded.source_address, None);
        assert_eq!(brake_message_decoded.trailer_abs_status, None);
        assert_eq!(
            brake_message_decoded.tractor_mounted_trailer_abs_warning_signal,
            None
        );
    }

    #[test]
    fn vehicle_electrical_power_message_1() {
        let electrical_power = VehicleElectricalPowerMessage::from_pdu(&[
            0xFF, 0xFF, 0xFF, 0xFF, 0xE3, 0x01, 0xE7, 0x01,
        ]);

        assert_eq!(electrical_power.net_battery_current, None);
        assert_eq!(electrical_power.alternator_current, None);
        assert_eq!(electrical_power.alternator_potential, None);
        assert_eq!(electrical_power.electrical_potential, Some(24));
        assert_eq!(electrical_power.battery_potential, Some(24));
    }

    #[test]
    fn vehicle_electrical_power_message_2() {
        let electrical_power_message_encoded = VehicleElectricalPowerMessage {
            net_battery_current: Some(-16),
            alternator_current: Some(5),
            alternator_potential: Some(235),
            electrical_potential: Some(1731),
            battery_potential: Some(947),
        }
        .to_pdu();
        let electrical_power_message_decoded =
            VehicleElectricalPowerMessage::from_pdu(&electrical_power_message_encoded);

        assert_eq!(
            electrical_power_message_decoded.net_battery_current,
            Some(-16)
        );
        assert_eq!(electrical_power_message_decoded.alternator_current, Some(5));
        assert_eq!(
            electrical_power_message_decoded.alternator_potential,
            Some(235)
        );
        assert_eq!(
            electrical_power_message_decoded.electrical_potential,
            Some(1731)
        );
        assert_eq!(
            electrical_power_message_decoded.battery_potential,
            Some(947)
        );
    }

    #[test]
    fn engine_fluid_level_pressure_2_message_1() {
        let engine_fluid = EngineFluidLevelPressure2Message::from_pdu(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);

        assert_eq!(engine_fluid.injection_control_pressure, Some(0));
        assert_eq!(engine_fluid.injector_metering_rail1_pressure, Some(0));
        assert_eq!(engine_fluid.injector_timing_rail1_pressure, Some(0));
        assert_eq!(engine_fluid.injector_metering_rail2_pressure, Some(0));
    }

    #[test]
    fn engine_fluid_level_pressure_2_message_2() {
        let engine_fluid_message_encoded = EngineFluidLevelPressure2Message {
            injection_control_pressure: Some(6),
            injector_metering_rail1_pressure: Some(81),
            injector_timing_rail1_pressure: None,
            injector_metering_rail2_pressure: Some(241),
        }
        .to_pdu();
        let engine_fluid_message_decoded =
            EngineFluidLevelPressure2Message::from_pdu(&engine_fluid_message_encoded);

        assert_eq!(
            engine_fluid_message_decoded.injection_control_pressure,
            Some(6)
        );
        assert_eq!(
            engine_fluid_message_decoded.injector_metering_rail1_pressure,
            Some(81)
        );
        assert_eq!(
            engine_fluid_message_decoded.injector_timing_rail1_pressure,
            None
        );
        assert_eq!(
            engine_fluid_message_decoded.injector_metering_rail2_pressure,
            Some(241)
        );
    }

    #[test]
    fn vehicle_position_not_available() {
        // All 0xFF = not available
        let msg = VehiclePositionMessage::from_pdu(&[0xFF; 8]);
        assert_eq!(msg.latitude, None);
        assert_eq!(msg.longitude, None);
    }

    #[test]
    fn vehicle_position_first_byte_0xff() {
        // First byte is 0xFF but the full 4-byte field is NOT all-0xFF,
        // so this should be treated as a valid value, not "not available"
        let pdu = [0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00];
        let msg = VehiclePositionMessage::from_pdu(&pdu);
        assert!(msg.latitude.is_some());
        assert!(msg.longitude.is_some());
    }

    #[test]
    fn vehicle_position_roundtrip() {
        let msg = VehiclePositionMessage {
            latitude: Some(52.0),
            longitude: Some(4.5),
        };
        let encoded = msg.to_pdu();
        let decoded = VehiclePositionMessage::from_pdu(&encoded);
        // f32 round-trip tolerance
        assert!((decoded.latitude.expect("latitude should decode") - 52.0).abs() < 0.01);
        assert!((decoded.longitude.expect("longitude should decode") - 4.5).abs() < 0.01);
    }

    #[test]
    fn vehicle_position_none_roundtrip() {
        let msg = VehiclePositionMessage {
            latitude: None,
            longitude: None,
        };
        let encoded = msg.to_pdu();
        assert_eq!(encoded, [0xFF; 8]);
        let decoded = VehiclePositionMessage::from_pdu(&encoded);
        assert_eq!(decoded.latitude, None);
        assert_eq!(decoded.longitude, None);
    }

    #[test]
    fn fuel_economy_not_available() {
        let msg = FuelEconomyMessage::from_pdu(&[0xFF; 8]);
        assert_eq!(msg.fuel_rate, None);
        assert_eq!(msg.instantaneous_fuel_economy, None);
        assert_eq!(msg.average_fuel_economy, None);
        assert_eq!(msg.throttle_position, None);
    }

    #[test]
    fn fuel_economy_first_byte_0xff() {
        // First byte of each 2-byte field is 0xFF but second is not,
        // so these should be treated as valid values
        let pdu = [0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0x50, 0xFF];
        let msg = FuelEconomyMessage::from_pdu(&pdu);
        assert!(msg.fuel_rate.is_some());
        assert!(msg.instantaneous_fuel_economy.is_some());
        assert!(msg.average_fuel_economy.is_some());
    }

    #[test]
    fn reset_message_roundtrip() {
        let msg = ResetMessage {
            trip_group_1: Some(true),
            trip_group_2_proprietary: Some(false),
            service_component_identification: Some(42),
            engine_build_hours_reset: Some(true),
            steering_straight_ahead_position_reset: Some(false),
            engine_spark_plug_secondary_voltage_tracking_reset: Some(true),
            engine_ignition_control_maintenance_hours_reset: Some(false),
            bin_lift_count_reset: Some(true),
            tire_configuration_information: Some(false),
            tire_sensor_information: Some(true),
        };
        let pdu = msg.to_pdu();
        let msg2 = ResetMessage::from_pdu(&pdu);
        assert_eq!(msg.trip_group_1, msg2.trip_group_1);
        assert_eq!(
            msg.trip_group_2_proprietary,
            msg2.trip_group_2_proprietary
        );
        assert_eq!(
            msg.service_component_identification,
            msg2.service_component_identification
        );
        assert_eq!(msg.engine_build_hours_reset, msg2.engine_build_hours_reset);
        assert_eq!(
            msg.steering_straight_ahead_position_reset,
            msg2.steering_straight_ahead_position_reset
        );
        assert_eq!(
            msg.engine_spark_plug_secondary_voltage_tracking_reset,
            msg2.engine_spark_plug_secondary_voltage_tracking_reset
        );
        assert_eq!(
            msg.engine_ignition_control_maintenance_hours_reset,
            msg2.engine_ignition_control_maintenance_hours_reset
        );
        assert_eq!(msg.bin_lift_count_reset, msg2.bin_lift_count_reset);
        assert_eq!(
            msg.tire_configuration_information,
            msg2.tire_configuration_information
        );
        assert_eq!(msg.tire_sensor_information, msg2.tire_sensor_information);
    }

    #[test]
    fn acknowledgment_message_roundtrip() {
        let msg = AcknowledgmentMessage {
            control_byte: Some(AcknowledgmentType::Negative),
            group_function_value: 0x80,
            pgn: crate::pgn::PGN::Tachograph,
        };
        let pdu = msg.to_pdu();
        let msg2 = AcknowledgmentMessage::from_pdu(&pdu);
        assert_eq!(msg.control_byte, msg2.control_byte);
        assert_eq!(msg.group_function_value, msg2.group_function_value);
        assert_eq!(msg.pgn, msg2.pgn);
    }

    #[test]
    fn packet_1cdeee17_roundtrip() {
        // 1CDEEE17#FCFFFFFFFFFFFFFF
        // PGN: 56832 (Reset)
        let data = [0xFC, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let msg = ResetMessage::from_pdu(&data);
        let encoded = msg.to_pdu();
        assert_eq!(data, encoded);
    }

    #[test]
    fn packet_1ce8ffee_roundtrip() {
        // 1CE8FFEE#00FFFFFFFF00DE00
        // PGN: 59392 (Acknowledgment Message)
        let data = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xDE, 0x00];
        let msg = AcknowledgmentMessage::from_pdu(&data);
        let encoded = msg.to_pdu();
        assert_eq!(data, encoded);
    }
}
