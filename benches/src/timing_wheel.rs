use chronographer::utils::ByteWheel;

#[divan::bench(args = [0u8, 1, 42, 128, 200, 255])]
fn insert_byte_wheel(bencher: divan::Bencher, pos: u8) {
    bencher.bench(|| {
        let mut wheel = ByteWheel::<usize>::default();
        wheel.insert(divan::black_box(pos), 1);
    });
}

#[divan::bench(args = [0u8, 1, 42, 128, 200, 255])]
fn skip_byte_wheel(bencher: divan::Bencher, to: u8) {
    bencher.bench(|| {
        let mut wheel = ByteWheel::<usize>::default();
        wheel.skip(divan::black_box(to));
    });
}

#[divan::bench]
fn tick_byte_wheel(bencher: divan::Bencher) {
    bencher.bench(|| {
        let mut wheel = ByteWheel::<usize>::default();
        divan::black_box(wheel.tick());
    });
}

#[divan::bench(args = [0usize, 10, 100, 1_000, 10_000])]
fn tick_byte_wheel_batch(bencher: divan::Bencher, count: usize) {
    bencher.counter(count).bench(|| {
        let mut wheel = ByteWheel::<usize>::default();
        for i in 0..count {
            wheel.insert(1, i);
        }
        let (expired, wrapped) = wheel.tick();
        divan::black_box((expired.len(), wrapped));
    });
}

#[divan::bench(args = [0usize, 10, 100, 1_000, 10_000])]
fn tick_byte_wheel_wrap(bencher: divan::Bencher, count: usize) {
    bencher.bench(|| {
        let mut wheel = ByteWheel::<usize>::default();
        for i in 0..count {
            wheel.insert(0, i);
        }
        wheel.skip(255);
        let (expired, wrapped) = wheel.tick();
        divan::black_box((expired.len(), wrapped));
    });
}

