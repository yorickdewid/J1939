use j1939::spn::*;

fn hex_to_bytes(s: &str) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate().take(8) {
        let hi = (chunk[0] as char).to_digit(16).expect("invalid hex digit");
        let lo = (chunk[1] as char).to_digit(16).expect("invalid hex digit");
        bytes[i] = u8::try_from((hi << 4) | lo).expect("hex byte fits in u8");
    }
    bytes
}

#[test]
fn time_date_roundtrip_from_dump() {
    // 0x18FEE6EE#243412024029837D (PGN 65254 - Time/Date)
    let data = hex_to_bytes("243412024029837D");
    let msg = TimeDate::from_pdu(&data);
    let encoded = msg.to_pdu();
    assert_eq!(encoded, data);
}

#[test]
fn vdhr_roundtrip_zero() {
    // 0x18FEC1EE#0000000000000000 (PGN 65217 - High Resolution Vehicle Distance)
    let data = [0u8; 8];
    let msg = HighResolutionVehicleDistanceMessage::from_pdu(&data);
    let encoded = msg.to_pdu();
    assert_eq!(encoded, data);
}

#[test]
fn tco1_roundtrip_from_dump() {
    // 0x0CFE6CEE#00FFFFC500000000 (PGN 65132 - Tachograph)
    let data = hex_to_bytes("00FFFFC500000000");
    let msg = TachographMessage::from_pdu(&data);
    let encoded = msg.to_pdu();
    assert_eq!(encoded, data);
}
