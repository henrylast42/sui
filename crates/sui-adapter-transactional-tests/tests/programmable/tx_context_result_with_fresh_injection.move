// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Under --allow-references-in-ptbs, a `&mut Y` returned from
// `borrow_mut(&mut X, &mut TxContext)` can be passed to a function that also takes
// `&mut TxContext` in the very next command. If the injected TxContext participated in
// the borrow graph, the result would mutably extend it and conflict with the fresh
// `&mut TxContext` injection in the same call.

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

public fun borrow_mut(x: &mut X, _ctx: &mut TxContext): &mut Y {
    &mut x.y
}

public fun write(y: &mut Y, _ctx: &mut TxContext) {
    y.f = y.f + 1;
}

public fun delete(x: X) {
    let X { id, y: Y { f: _ } } = x;
    object::delete(id);
}

//# programmable
//> 0: test::m::new();
//> 1: test::m::borrow_mut(Result(0));
//> 2: test::m::write(Result(1));
//> 3: test::m::delete(Result(0));
