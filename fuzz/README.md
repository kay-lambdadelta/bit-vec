# fuzzer for bit-vec, bit-set and bit-matrix

Based on fuzzing in `smallvec`.

# fuzzing

```sh
cargo afl build --release --features afl && cargo afl fuzz -i in -o out ../target/release/bit-fuzz
```

It may ask you to prepare the Linux system by running commands such as these:
```sh
echo core | sudo tee /proc/sys/kernel/core_pattern
cd /sys/devices/system/cpu
echo performance | sudo tee cpu*/cpufreq/scaling_governor
```
