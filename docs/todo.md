# Todo

Improvements found during the Rust port. The port itself changes no behaviour; these wait for after
cutover.

- A rename by clicking a binder redraws the term at once: a wider name shifts what follows in one
  frame. It could run as an α beat (`Bench::beat` with `Kind::Alpha` at the binder's path), as the
  automatic α does.
