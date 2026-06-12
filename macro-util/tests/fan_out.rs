#[test]
fn test_fan_out() {
    macro_rules! target_macro {
        (
            [prefix tokens]
            tuple_macro!(
                [prefix tokens]
                tuple_macro!(
                    [prefix tokens]
                    tuple_macro!(
                        [prefix tokens]
                        item_macro!([prefix tokens] Ident0(args 0)),
                        item_macro!([prefix tokens] Ident1(args 1)),
                    ),
                    tuple_macro!(
                        [prefix tokens]
                        item_macro!([prefix tokens] Ident2(args 2)),
                        item_macro!([prefix tokens] Ident3(args 3)),
                    ),
                ),
                tuple_macro!(
                    [prefix tokens]
                    tuple_macro!(
                        [prefix tokens]
                        item_macro!([prefix tokens] Ident4(args 4)),
                        item_macro!([prefix tokens] Ident5(args 5)),
                    ),
                    tuple_macro!(
                        [prefix tokens]
                        item_macro!([prefix tokens] Ident6(args 6)),
                    ),
                ),
            );
            {
                Ident0(args 0) (p0 p0 p0),
                Ident1(args 1) (p0 p0 p1),
                Ident2(args 2) (p0 p1 p0),
                Ident3(args 3) (p0 p1 p1),
                Ident4(args 4) (p1 p0 p0),
                Ident5(args 5) (p1 p0 p1),
                Ident6(args 6) (p1 p1 p0),
            }
        ) => {
            println!("it works");
        };
    }

    traffloat_macro_util::fan_out! {
        [prefix tokens]
        target_macro, tuple_macro, item_macro;
        2, 3;
        Ident0(args 0),
        Ident1(args 1),
        Ident2(args 2),
        Ident3(args 3),
        Ident4(args 4),
        Ident5(args 5),
        Ident6(args 6),
    }
}
