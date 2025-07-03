let { WorkspaceMember, .. } = import "contracts.ncl"
in
(import "workspace-crate/Cargo.toml") | WorkspaceMember
