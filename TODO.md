# Release 0.5.0
    - Finish and update all the benchmark graphics in lexical/assets.
    - Run the comprehensive suite
    - Need to see if I can further optimize codegen.
        - Likely doing too many comparisons

- Optimize integer formatting using:
    - https://gist.github.com/Veedrac/069dd35f896de4df4e14b881a455ca47

- Port Knuth's Algorithm D to Rust's libcore.
- target_arch = "..."
    - Use 64-bit limbs on x86_64, mips64, aarch64, and powerpc64, but not sparc64.

- There's a lot of work to do in bignum, we need to make sure we have proper assumptions.


- Optimize slices.
    - Use a SliceCursor type to simplify this.
        - Likely just change to a cursor or span name.
    - Maybe use unchecked_index
        - It really should have all the features of a slice though...
            - I really want the features of a regular type...
    - This would dramatically simplify my code, and prevent errors with new assumptions.
        - Just use slices, but make a slice-like type without index checking.


# Performance Issues
    - check atof_real with performance governor on a fresh boot.
        sudo cpupower frequency-set -g performance
        cargo bench atof_real

        - Yep, the codegen seems to suck for my functions, why?
            - Line 46 exemplifies the problem, about 20% slower.
                - It's not the 0s that are the issue, it's the non-zeros.
                    - 0-20
                        test atof_real_f64_lexical ... bench:     234,988 ns/iter (+/- 25,788)
                        test atof_real_f64_parse   ... bench:     308,434 ns/iter (+/- 8,204)
                    - 20-40
                        test atof_real_f64_lexical ... bench:     325,983 ns/iter (+/- 11,390)
                        test atof_real_f64_parse   ... bench:     352,760 ns/iter (+/- 24,026)
                    - 40-60
                        test atof_real_f64_lexical ... bench:     564,150 ns/iter (+/- 16,675)
                        test atof_real_f64_parse   ... bench:     433,423 ns/iter (+/- 14,140)
                    - 60-80
                        test atof_real_f64_lexical ... bench:     659,454 ns/iter (+/- 22,818)
                        test atof_real_f64_parse   ... bench:     445,325 ns/iter (+/- 56,434)
                    - 80-100
                    - 100-120
                    - 120-140

        - Going to need to speed up the fast path, which likely means parse_mantissa.
            - Almost all of the time is in parse_mantissa, so...
                - We need to actually focus on there...

# x87 without SSE2.
    - Going to need inline assembly, as per the Rust documentation.
