// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Under --allow-references-in-ptbs, a call taking multiple TxContext parameters
// (`&mut` and `&`) alongside an object still roots its returned reference only in the
// object. The result stays valid across a later mutable TxContext use. This is the
// hardest agreement case for the invariant checker, which fully tracks each injection
// as a distinct borrow root.

//# init --addresses test=0x0 --allow-references-in-ptbs

//# publish
module test::m;

public struct X has key, store {
    id: UID,
    y: Y,
}

public struct Y has store {
    f: u64,
}

public fun new(ctx: &mut TxContext): X {
    X { id: object::new(ctx), y: Y { f: 0 } }
}

public fun borrow_mut2(x: &mut X, _a: &mut TxContext, _b: &TxContext): &mut Y {
    &mut x.y
}

public fun write(y: &mut Y, _ctx: &mut TxContext) {
    y.f = y.f + 1;
}

public fun mut_tx(_: &mut TxContext) {
}

public fun check(y: &Y, expected: u64, _ctx: &TxContext) {
    assert!(y.f == expected, 0);
}

public fun delete(x: X) {
    let X { id, y: Y { f: _ } } = x;
    object::delete(id);
}

//# programmable --inputs 1
//> 0: test::m::new();
//> 1: test::m::borrow_mut2(Result(0));
//> 2: test::m::mut_tx();
//> 3: test::m::write(Result(1));
//> 4: test::m::check(Result(1), Input(0));
//> 5: test::m::delete(Result(0));
