fn main() {
    assert_eq!(f32::from_bits(0x0000_0000), 0f32);
    assert_eq!(f32::from_bits(0x8000_0000), 0f32);
    assert_eq!(f32::from_bits(0x7F7F_FFFF), f32::MAX);
    assert_eq!(f32::from_bits(0xFF7F_FFFF), f32::MIN);
    assert_eq!(f32::from_bits(0x0080_0000), f32::MIN_POSITIVE);
    assert_eq!(f32::from_bits(0x7F80_0000), f32::INFINITY);
    assert_eq!(f32::from_bits(0xFF80_0000), f32::NEG_INFINITY);

    //  0.125 0,01111100,0... -> 0,01111110,01...
    //  0.5   0,01111110,0... == 0,01111110,0...
    //  0.625                  + 0,01111110,010...
    // -0.375                  - 1,01111101,10...
    let one_eighth = f32::from_bits(0x3E00_0000);
    let half = f32::from_bits(0x3F00_0000);
    let five_eighths = f32::from_bits(0x3F20_0000);
    let negtive_three_eighths = f32::from_bits(0xBEC0_0000);
    assert_eq!(one_eighth, 0.125f32);
    assert_eq!(half, 0.5f32);
    assert_eq!(one_eighth + half, five_eighths);
    assert_eq!(one_eighth - half, negtive_three_eighths);

    let lhs = f32::from_bits(0x4128_0000);
    let rhs = f32::from_bits(0x42F1_4000);
    assert_eq!(lhs, 10.5f32);
    assert_eq!(rhs, 120.625f32);
    assert_eq!(lhs - rhs, f32::from_bits(0xC2DC_4000));

    assert!(f32::from_bits(0xFF80_AE86).is_nan());
    assert!(f32::from_bits(0x7F80_AE86).is_nan());
}
