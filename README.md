# The Most Overengineered Calculator

A ridiculously overengineered, memory-safe, compiler-inspired CLI calculator written in Rust.

## Run the current CLI

Pass two signed 64-bit operands and an operator as three arguments. Quote
`*` to keep the shell from expanding it.

```sh
cargo run --locked --bin overcalc -- 1 + 1    # 2
cargo run --locked --bin overcalc -- -7 / 3   # -2
cargo run --locked --bin overcalc -- 6 '*' 7  # 42
```

Success writes one result line to stdout and exits with status 0. Invalid
arguments, division by zero, and overflow write one diagnostic to stderr
and exit with status 1.

## Planned CLI

```sh
overcalc "1 + 1"                      # 2
overcalc "1 + 1" --engine naive       # 2
overcalc "1 + 1" --engine enterprise  # 2
overcalc benchmark
```

The arithmetic contract uses checked signed 64-bit integers, parentheses, unary minus, and the four basic operators. Division truncates toward zero. Division by zero and overflow produce errors.

## Roadmap

Each milestone gets its own pull request.

- [x] **1. Arithmetic contract and CLI.** Set up the Cargo workspace and CI. Specify integer boundaries, error output, and exit codes so every engine has the same observable behavior.
- [ ] **2. Compiler frontend.** Build a zero-copy lexer, Pratt parser, arena-backed AST, semantic analysis, and a direct evaluator. Borrowed input and byte spans let diagnostics point to the original expression while node IDs expose the arena's storage model.
- [ ] **3. IR, bytecode, and VM.** Lower the AST to linear IR, emit register bytecode, and execute it in the enterprise engine. Separate representations make it possible to measure compilation and repeatedly execute a compiled program.
- [ ] **4. Optimization passes.** Implement constant folding and compare instruction counts. Failed folds stay at their original execution positions so the first reported arithmetic error survives optimization.
- [ ] **5. Memory and allocation tracking.** Reuse preallocated VM registers and count heap allocations. Target zero allocations during repeated execution of an already compiled program with initialized buffers.
- [ ] **6. Verification.** Add property tests, fuzzing, Miri, and CLI integration tests. Compare both engines and use explicit expected results to check the shared frontend.
- [ ] **7. Benchmarks and published results.** Add Criterion benchmarks, the benchmark command, and architecture and measurement docs. Measure end-to-end evaluation, compilation, and VM execution with and without optimization. Record the environment and report measured results.

## Pull request reviews

After implementation, review each PR in a separate verification pass against its acceptance criteria and diff. Record:

1. Verification commands, observed results, and remaining gaps.
2. The intentionally added bottleneck or implementation complexity and why it is worth studying. Mark predicted costs as hypotheses until measured.
3. A technically defensible way to make the design harder, including the behavior or invariant it would let us test.
4. Whether that design belongs in the current PR or a later milestone, and which correctness issues were fixed.
