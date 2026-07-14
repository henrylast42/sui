// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Under --allow-references-in-ptbs, a `&mut TxContext` returned from a call has an empty
// source set in the borrow graph (the injected TxContext is carved out). Such a rootless
// reference cannot be used anywhere (supplying TxContext manually is rejected), so this
// checks it is inert: it conflicts with nothing and is released cleanly at the end of the
// transaction.

//# init --addresses test=0x0 --allow-references-in-ptbs

//# publish
module test::m;

public fun mut_id(ctx: &mut TxContext): &mut TxContext {
    ctx
}

public fun mut_tx(_: &mut TxContext) {
}

public fun eq_digests(a: &vector<u8>, b: &vector<u8>) {
    assert!(*a == *b, 0);
}

//# programmable
//> 0: test::m::mut_id();
//> 1: test::m::mut_tx();
//> 2: sui::tx_context::digest();
//> 3: sui::tx_context::digest();
//> 4: test::m::eq_digests(Result(2), Result(3));
