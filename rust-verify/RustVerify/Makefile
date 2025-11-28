
RustVerify/Sandbox.lean : ../sandbox/sandbox.llbc
	aeneas -backend lean ../sandbox/sandbox.llbc -dest RustVerify

../sandbox/sandbox.llbc : ../sandbox/src/main.rs
	cd ../sandbox && charon cargo --preset=aeneas