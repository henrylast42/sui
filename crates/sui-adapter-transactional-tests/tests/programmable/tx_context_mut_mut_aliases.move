// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// two `&mut TxContext` resolve to the same runtime TxContext; digests match.

//# init --addresses test=0x0 --allow-references-in-ptbs

//# publish
module test::m;

use sui::tx_context::digest;

public fun mut_mut_observe(a: &mut TxContext, b: &mut TxContext) {
    let da = *digest(a);
    let db = *digest(b);
    assert!(da == db, 0);
}

//# programmable
//> test::m::mut_mut_observe();
