import Lake
open System Lake DSL

package simple

-- this will probably require your git client to be authenticated.
require «lean-sts» from git "git@github.com:verse-lab/lean-sts.git"@"main"

@[default_target]
lean_lib Simple where
  defaultFacets := #[LeanLib.sharedFacet]

@[default_target]
lean_lib Globals where
  defaultFacets := #[LeanLib.sharedFacet]

@[default_target]
lean_lib Arrays where
  defaultFacets := #[LeanLib.sharedFacet]

@[default_target]
lean_lib Structs where
  defaultFacets := #[LeanLib.sharedFacet]
