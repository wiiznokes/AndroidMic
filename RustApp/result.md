# Linux, master

bench_resampling time: [22.339 µs 22.393 µs 22.447 µs]

# Linux, rubato 0.16, opti no copy input

bench_resampling time: [19.860 µs 20.463 µs 21.003 µs]
change: [−9.3247% −7.3958% −5.7062%] (p = 0.00 < 0.05)
Performance has improved.

# Linux, rubato 2.0

bench_resampling time: [15.149 µs 15.495 µs 15.876 µs]
change: [−24.540% −22.301% −20.162%] (p = 0.00 < 0.05)
Performance has improved.

## For speexdsp

# Linux, master

bench_speexdsp time: [97.000 µs 97.257 µs 97.523 µs]

## For process

# Linux, after resampling cache result opti

bench_process time: [160.59 µs 162.31 µs 164.43 µs]

## Opti plan

- [ ] test ringbuffer instead of rtrb
- [ ] add optimization flag when vendoring speexdsp
- [ ] speexdsp
- [ ] rnnoise
- [ ] writting to the ring buff


- https://crates.io/crates/ringbuf
- https://crates.io/crates/ringbuffer
- https://crates.io/crates/rtrb