#![cfg(test)]

use traffloat_types::time::Rate;
use traffloat_types::units::LiquidVolume;

use crate::{Config, Scenario, State, TfsaveFile};

#[test]
fn test_write_parse() {
    let schema = TfsaveFile::builder()
        .scenario(
            Scenario::builder()
                .name(arcstr::literal!("foo"))
                .description(arcstr::literal!("bar"))
                .build(),
        )
        .config(Config::builder().sun_speed(Rate(1.)).negligible_volume(LiquidVolume(1.)).build())
        .def(Vec::new())
        .state(State::default())
        .build();

    let mut buf = Vec::new();
    schema.write(&mut buf).expect("<Vec<u8> as Write> is infallible");

    TfsaveFile::parse(&buf).expect("Error parsing freshly written schema");
}
