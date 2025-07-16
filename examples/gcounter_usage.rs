use cmrdt::{ActorId, AddCtx, CmRDT, Dot, GCounter, VClock};

fn main() {
    // 1. We have two servers logging page views.
    let server1_id = ActorId(1);
    let server2_id = ActorId(2);

    // 2. Each server gets its own GCounter replica.
    let mut server1_views = GCounter::default();
    let mut server2_views = GCounter::default();
    println!("Initial total views: {}", server1_views.read()); // => 0

    // 3. Server 1 logs 10 page views.
    let server1_op_ctx = AddCtx {
        dot: Dot {
            actor: server1_id,
            counter: 1,
        },
        clock: VClock::default(),
    };
    server1_views.apply(cmrdt::gcounter::Op::Inc(10), server1_op_ctx);
    println!("Views on Server 1: {}", server1_views.read()); // => 10

    // 4. Concurrently, Server 2 logs 15 page views.
    let server2_op_ctx = AddCtx {
        dot: Dot {
            actor: server2_id,
            counter: 1,
        },
        clock: VClock::default(),
    };
    server2_views.apply(cmrdt::gcounter::Op::Inc(15), server2_op_ctx);
    println!("Views on Server 2: {}", server2_views.read()); // => 15

    // 5. Now, we sync the servers by merging Server 2's data into Server 1.
    server1_views.merge(server2_views);

    // 6. The final value is the sum of all views from all servers.
    println!("Final total views after merge: {}", server1_views.read()); // => 25
}
