# My solutions to [Shuttle's Christmas Code Hunt 2024](https://www.shuttle.dev/cch)

Bonus tasks not included.

## Interesting implementations

- [Day 2](./src/days/day_2.rs): shows the terseness that can be achieved with salvo. Note that these routes are documented with OpenAPI too. Though, as the other examples show, it often takes a lot more characters to implement custom inputs, outputs and errors.
- [Day 5](https://github.com/Samyak2/shuttlings-cch24/blob/8ef580277380be894e3f8a172d1996e902287c53/src/days/day_5.rs#L95-L134): shows error handling in salvo.
- [Day 9](./src/days/day_2.rs): uses atomics and shows usage of global state in salvo.
    - Uses a custom implementation of saturating subtraction/addition of an atomic value. Since there's no built-in method for this (the `fetch_sub` method overflows), it is implemented using a [CAS](https://en.wikipedia.org/wiki/Compare-and-swap) loop.

## A note on the dependencies

- `salvo`: I'm currently using my fork of salvo due to [salvo#1014](https://github.com/salvo-rs/salvo/pull/1014) and [salvo#1015](https://github.com/salvo-rs/salvo/pull/1015). It will be updated to the latest version once those fixes are merged.
- `shuttle-{salvo,runtime,shared-db}`: the shuttle libraries are also from my fork of shuttle. This is because shuttle-salvo currently uses salvo v0.63 (see [shuttle#1941](https://github.com/shuttle-hq/shuttle/issues/1941)). These will be updated once shuttle uses the latest version of salvo with the above fixes.
