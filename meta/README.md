# The meta folder

This folder is ignored via the `.genignore` file. It contains meta files
that should not make it into the generated project.

In particular, it is used for an appveyor CI script that runs on `cosmwasm-template`
itself (running the cargo-generate script, then testing the generated project).
The `.circleci` directory contains a script destined for any projects created from
this template.
