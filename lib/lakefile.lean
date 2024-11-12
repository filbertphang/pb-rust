import Lake
open System Lake DSL

-- TODO: see if this part is still necessary after the implementing fat static library build
-- not sure why I have to link with libstdc++ here, since everything that
-- uses libstdc++ is a dependency of `lean-sts` (which should already link with libstdc++)
def libcpp : String :=
  if System.Platform.isWindows then "libstdc++-6.dll"
  else if System.Platform.isOSX then "libc++.dylib"
  else "libstdc++.so.6"

package LeanRB where
  moreLeanArgs := #[s!"--load-dynlib={libcpp}"]
  moreGlobalServerArgs := #[s!"--load-dynlib={libcpp}"]
  moreLinkArgs :=
    if System.Platform.isOSX || System.Platform.isWindows then #[]
    else #["-L/usr/lib/x86_64-linux-gnu", "/usr/lib/x86_64-linux-gnu/libstdc++.so.6"]

-- this will probably require your git client to be authenticated.
require «lean-sts» from git "git@github.com:verse-lab/lean-sts.git"@"main"
-- require «lean-sts» from git "git@github.com:filbertphang/lean-sts.git"@"main"

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

@[default_target]
lean_lib ReliableBroadcast where
  defaultFacets := #[LeanLib.sharedFacet]

-- build this module as a static fat library.
-- i.e. package all dependencies and make those symbols available within this lib.
-- references this PR: https://github.com/leanprover/lean4/pull/4271/files
-- and this accompanying zulip thread: https://leanprover.zulipchat.com/#narrow/channel/270676-lean4/topic/reverse.20FFI.3A.20building.20a.20.22fat.22.20static.20library.3F

/-- The path to the static fat library in the package's `libDir`. -/
@[inline] def fatStaticFile (self : LeanLib) : FilePath :=
  self.pkg.nativeLibDir / nameToStaticLib s!"{self.config.libName}Fat"

@[specialize] protected def LeanLib.buildFatStatic
(self : LeanLib) : FetchM (BuildJob FilePath) := do
  withRegisterJob s!"{self.name}:static.fat" do
  let mods ← (← self.modules.fetch).concatMapM fun mod => do
    return (← mod.transImports.fetch).push mod
  let oJobs ← mods.concatMapM fun mod =>
    mod.nativeFacets (shouldExport := false) |>.mapM fun facet => fetch <| mod.facet facet.name
  let libFile := fatStaticFile self
  IO.println s!"successfully built: {libFile}"
  buildStaticLib libFile oJobs

library_facet fatStatic (lib : LeanLib) : FilePath :=
  LeanLib.buildFatStatic lib

@[default_target]
lean_lib «Protocol» {
  defaultFacets := #[`fatStatic]
}
