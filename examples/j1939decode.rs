use std::env;

use j1939::Id;
use j1939::PGN;
use j1939::diagnostic;
use j1939::protocol;
use j1939::spn::{
    AmbientConditionsMessage, CabIlluminationMessage, ECUHistoryMessage,
    ElectronicBrakeController1Message, ElectronicEngineController1Message,
    ElectronicEngineController2Message, ElectronicEngineController3Message,
    EngineFluidLevelPressure1Message, EngineFluidLevelPressure2Message,
    EngineTemperature1Message, FanDriveMessage, FuelConsumptionMessage, FuelEconomyMessage,
    HighResolutionVehicleDistanceMessage, InletExhaustConditions1Message,
    PowerTakeoffInformationMessage, ShutdownMessage, TachographMessage, TankInformation1Message,
    TimeDate, TorqueSpeedControl1Message, VehicleDistanceMessage, VehicleElectricalPowerMessage,
    VehiclePositionMessage,
};

fn usage() {
    println!("Usage: j1939decode <input>");
    println!();
    println!("Options:");
    println!("  <input>     29-bit CAN ID in hexadecimal format (0x18EAFF00)");
    println!("              or CAN ID and data separated by '#' (0x18FEE6EE#243412024029837D)");
}

fn decode_data(pgn: PGN, data: &[u8]) {
    println!("Data Decoded:");
    match pgn {
        PGN::TorqueSpeedControl1 => {
            println!("  {}", TorqueSpeedControl1Message::from_pdu(data));
        }
        PGN::ElectronicEngineController1 => {
            println!("  {}", ElectronicEngineController1Message::from_pdu(data));
        }
        PGN::ElectronicEngineController2 => {
            println!("  {}", ElectronicEngineController2Message::from_pdu(data));
        }
        PGN::ElectronicEngineController3 => {
            println!("  {}", ElectronicEngineController3Message::from_pdu(data));
        }
        PGN::ElectronicBrakeController1 => {
            println!("  {}", ElectronicBrakeController1Message::from_pdu(data));
        }
        PGN::AmbientConditions => {
            println!("  {}", AmbientConditionsMessage::from_pdu(data));
        }
        PGN::VehiclePosition => {
            println!("  {}", VehiclePositionMessage::from_pdu(data));
        }
        PGN::FuelEconomy => {
            println!("  {}", FuelEconomyMessage::from_pdu(data));
        }
        PGN::EngineFluidLevelPressure1 => {
            println!("  {}", EngineFluidLevelPressure1Message::from_pdu(data));
        }
        PGN::FuelConsumption => {
            println!("  {}", FuelConsumptionMessage::from_pdu(data));
        }
        PGN::VehicleDistance => {
            println!("  {}", VehicleDistanceMessage::from_pdu(data));
        }
        PGN::HighResolutionVehicleDistance => {
            println!("  {}", HighResolutionVehicleDistanceMessage::from_pdu(data));
        }
        PGN::FanDrive => {
            println!("  {}", FanDriveMessage::from_pdu(data));
        }
        PGN::Shutdown => {
            println!("  {}", ShutdownMessage::from_pdu(data));
        }
        PGN::EngineTemperature1 => {
            println!("  {}", EngineTemperature1Message::from_pdu(data));
        }
        PGN::InletExhaustConditions1 => {
            println!("  {}", InletExhaustConditions1Message::from_pdu(data));
        }
        PGN::VehicleElectricalPower1 => {
            println!("  {}", VehicleElectricalPowerMessage::from_pdu(data));
        }
        PGN::EngineFluidLevelPressure2 => {
            println!("  {}", EngineFluidLevelPressure2Message::from_pdu(data));
        }
        PGN::AuxiliaryInputOutputStatus => {
            println!("  {}", CabIlluminationMessage::from_pdu(data));
        }
        PGN::ECUHistory => {
            println!("  {}", ECUHistoryMessage::from_pdu(data));
        }
        PGN::TANKInformation1 => {
            println!("  {}", TankInformation1Message::from_pdu(data));
        }
        PGN::Tachograph => {
            println!("  {}", TachographMessage::from_pdu(data));
        }
        PGN::PowerTakeoffInformation => {
            println!("  {}", PowerTakeoffInformationMessage::from_pdu(data));
        }
        PGN::DiagnosticMessage1 => {
            println!("  {}", diagnostic::Message1::from_pdu(data));
        }
        PGN::Request => {
            println!("  Request PGN: {:?}", protocol::request_from_pdu(data));
        }
        PGN::TimeDate => {
            // TimeDate currently uses Debug formatting for its decoded representation,
            // unlike other messages in this function that use Display. This is
            // intentional because TimeDate does not provide a custom Display format.
            println!("  {:?}", TimeDate::from_pdu(data));
        }
        _ => {
            println!("  Unknown PGN for data decoding.");
        }
    }
}

fn main() {
    let Some(input_str) = env::args().nth(1) else {
        usage();
        return;
    };

    if matches!(input_str.as_str(), "-h" | "--help") {
        usage();
        return;
    }

    let parts: Vec<&str> = input_str.split('#').collect();

    let id_str = parts[0];
    let id_raw = if id_str.starts_with("0x") {
        u32::from_str_radix(id_str.trim_start_matches("0x"), 16).expect("Invalid ID")
    } else {
        u32::from_str_radix(id_str, 16).expect("Invalid ID")
    };

    let id = Id::new(id_raw);

    println!("ID");
    println!(" Hex: 0x{:08X}", id.as_raw());
    println!(" Dec: {}", id.as_raw());
    println!(" Bin: {:029b}", id.as_raw());
    println!("Priority: {}", id.priority());
    println!("Data Page (DP): {}", id.data_page());
    println!("Parameter Group Number (PGN): {:?}", id.pgn());
    println!(" Hex: 0x{:04X}", id.pgn_raw());
    println!(" Dec: {}", id.pgn_raw());
    println!("PDU Format: {:?}", id.pdu_format());
    println!("Broadcast: {}", id.is_broadcast());

    if let Some(ge) = id.group_extension() {
        println!("Group Extension (GE)/PDU Specific (PS): 0x{ge:02X} ({ge})");
    }

    if let Some(da) = id.destination_address() {
        println!("Destination Address (DA): 0x{da:02X} ({da})");
    }

    let sa = id.source_address();
    println!("Source Address (SA): 0x{sa:02X} ({sa})");

    if parts.len() > 1 {
        let data_str = parts[1];

        if data_str.len() % 2 != 0 {
            eprintln!("Invalid data: hex string must have an even number of characters");
            return;
        }

        let mut data = Vec::new();
        for i in (0..data_str.len()).step_by(2) {
            let byte = u8::from_str_radix(&data_str[i..i + 2], 16).expect("Invalid data byte");
            data.push(byte);
        }
        println!();
        println!("Data Hex: {data:02X?}");
        if !data.is_empty() {
            decode_data(id.pgn(), &data);
        }
    }
}
