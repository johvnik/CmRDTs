use cmrdt::{ActorId, AddCtx, CmRDT, Dot, PNCounter, VClock};

fn main() {
    // 1. Define unique IDs for our two replicas (Alice and Bob)
    let alice_id = ActorId(1);
    let bob_id = ActorId(2);

    // 2. Create two separate counters, one for Alice and one for Bob
    let mut alice_counter = PNCounter::default();
    let mut bob_counter = PNCounter::default();
    println!("Initial state: {}", alice_counter.read()); // => 0

    // 3. Alice increments the counter by 5
    // In a real system, the VClock would be managed by a wrapper structure.
    // For this example, we create it manually.
    let alice_op_ctx = AddCtx {
        dot: Dot {
            actor: alice_id,
            counter: 1,
        },
        clock: VClock::default(),
    };
    alice_counter.apply(cmrdt::pn_counter::Op::Inc(5), alice_op_ctx);
    println!("After Alice increments by 5: {}", alice_counter.read()); // => 5

    // 4. Bob increments by 2 and decrements by 1
    let bob_op_1_ctx = AddCtx {
        dot: Dot {
            actor: bob_id,
            counter: 1,
        },
        clock: VClock::default(),
    };
    let bob_op_2_ctx = AddCtx {
        dot: Dot {
            actor: bob_id,
            counter: 2,
        },
        clock: VClock::default(),
    };
    bob_counter.apply(cmrdt::pn_counter::Op::Inc(2), bob_op_1_ctx);
    bob_counter.apply(cmrdt::pn_counter::Op::Dec(1), bob_op_2_ctx);
    println!("Bob's counter value: {}", bob_counter.read()); // => 1 (2 - 1)

    // 5. Now, let's merge Bob's changes into Alice's counter
    alice_counter.merge(bob_counter);

    // 6. Read the final, converged value
    println!("Final value after merge: {}", alice_counter.read()); // => 6 (5 + 1)
}
