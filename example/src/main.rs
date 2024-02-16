#[enum_range]
#[repr(u16)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
enum RangedEnum {
    Zero = 0,
    One = 1,
    #[range(
        format = "PrivateUse{index}_{value}",
        start = 10,
        end = 20,
        range_check = "is_private_use"
    )]
    PrivateUse,
    Three = 3,
    #[range(
        format = "Unassigned{value}",
        start = 200,
        end = 205,
        range_check = "is_unassigned"
    )]
    Unassigned,
    #[range(
        format = "WellKnown{index}",
        start = 206,
        end = 210,
        range_check = "is_well_known"
    )]
    WellKnown,
    Mdr = 400,
    Lol = 401,
    Ptdr = 402,
}

use enum_range::enum_range;

fn main() {
    let zero = RangedEnum::Zero;
    println!("RangedEnum::Zero is_well_known => {:?}", RangedEnum::is_well_known(zero));
    println!("RangedEnum::Zero is_unassigned => {:?}", RangedEnum::is_unassigned(zero));
    println!("RangedEnum::Zero is_private_use => {:?}", RangedEnum::is_private_use(zero));

    let pu10 = RangedEnum::PrivateUse0_10;
    println!("RangedEnum::PrivateUse0_10 is_well_known => {:?}", RangedEnum::is_well_known(pu10));
    println!("RangedEnum::PrivateUse0_10 is_unassigned => {:?}", RangedEnum::is_unassigned(pu10));
    println!("RangedEnum::PrivateUse0_10 is_private_use => {:?}", RangedEnum::is_private_use(pu10));
}
