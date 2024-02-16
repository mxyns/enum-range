# enum-range

Helps declaring multiple variants in an enum with discriminator.
Say you want an enum with discriminators represented by an u16 but you don't want to define every 
enum variant by hand. You can use `#[enum-range]` do to it for you.

# Example

Representing the ["BMP Message Types"](https://www.iana.org/assignments/bmp-parameters/bmp-parameters.xhtml) IANA Registry.

```rust
#[enum_range]
#[repr(u16)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
enum RangedEnum {
    RouteMonitoring = 0,
    StatisticsReport = 1,
    PeerDownNotification = 2,
    PeerUpNotification = 3,
    Initiation = 4,
    Termination = 5,
    #[range(
        format = "Unassigned{value}",
        start = 7,
        end = 250,
        range_check = "is_unassigned"
    )]
    Unassigned,
    #[range(
        format = "Experimental{index}_{value}",
        start = 251,
        end = 254,
        range_check = "is_experimental_use"
    )]
    Experimental,
    Reserved = 255
}

fn main() {
    assert!(RangedEnum::Unassigned7 as u16 == 7);
    assert!(RangedEnum::Unassigned250 as u16 == 250);
    assert!(RangedEnum::is_unassigned(RangedEnum::Unassigned7));
    assert!(false == RangedEnum::is_experimental_use(RangedEnum::Unassigned7));

    assert!(false == RangedEnum::is_experimental_use(RangedEnum::Initiation));
    assert!(false == RangedEnum::is_experimental_use(RangedEnum::Initiation));
}
 ```

For a detailed example check out the [example project](https://github.com/mxyns/enum-range/tree/master/example) on this crates' [repo](https://github.com/mxyns/enum-range/).
For details on the use of `#[enum_range]` and `#[range(...)]` macro see the Rust documentation.