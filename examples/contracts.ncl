let cargo_packages = (import "../Cargo.toml").workspace.members in
{
  Semver
    | doc "a string that is a valid semantic version: 99.99.99"
    = fun label ver =>
      if std.string.is_match "\\d+\\.\\d+\\.\\d+" ver then
        ver
      else
        std.contract.blame_with_message "value must be a valid semver string" label,
  Package
    | doc "a valid cargo workspace package defined by the name 'Cargo.toml'"
    = fun label pkg =>
      # if `cargo_packages` contains `pkg`
      if (cargo_packages |> std.array.any (fun member => member == pkg)) then
        pkg
      else
        std.contract.blame_with_message "must be a Cargo.toml workspace member" label,
  WorkspaceMember
    | doc "a cargo crate manifest where version and edition point to a workspace manifest"
    =
      std.contract.from_validator (fun cargo =>
        if cargo.package.version == { workspace = true } then
          'Ok
        else
          'Error {
            message = "%{cargo.package.name} is invalid",
            notes = ["version must be { workspace = true }"],
          }
      )
}
