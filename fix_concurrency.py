import pathlib
p = pathlib.Path("crates/uor-r4-object-store/tests/peer.rs"); s = p.read_text()
old = """    let pull = |mut receiver: Store, payload: Vec<u8>| {
        let stream = TcpStream::connect(addr).expect("connect");
        let mut reader = stream.try_clone().expect("clone");
        let mut writer = stream;
        let (pulled, stats) = peer::pull_expected(&mut receiver, Some(expected), &mut reader, &mut writer)
            .expect("concurrent pull");
        assert!(stats.verified);
        assert_eq!(receiver.rebuild(&pulled).expect("rebuild").bytes(), &payload[..]);
        stats
    };
    let a = thread::spawn(move || pull(receiver_a, payload_a));
    let b = thread::spawn(move || pull(receiver_b, payload_b));"""
new = """    fn pull_remote(
        addr: std::net::SocketAddr,
        expected: ExpectedObject,
        mut receiver: Store,
        payload: Vec<u8>,
    ) -> peer::PullStats {
        let stream = TcpStream::connect(addr).expect("connect");
        let mut reader = stream.try_clone().expect("clone");
        let mut writer = stream;
        let (pulled, stats) =
            peer::pull_expected(&mut receiver, Some(expected), &mut reader, &mut writer)
                .expect("concurrent pull");
        assert!(stats.verified);
        assert_eq!(
            receiver.rebuild(&pulled).expect("rebuild").bytes(),
            &payload[..]
        );
        stats
    }
    let a = thread::spawn(move || pull_remote(addr, expected, receiver_a, payload_a));
    let b = thread::spawn(move || pull_remote(addr, expected, receiver_b, payload_b));"""
assert old in s
s = s.replace(old, new, 1)
p.write_text(s)
print("nested fn refactor applied")