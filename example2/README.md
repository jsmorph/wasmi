This example is a freeze/thaw variant of `../example`.  Sadly this
gear does not support serialization, so resuming the WASM execution
can't be done in isolation.  One implication is that we couldn't
ZK-execute each pure WASM computation.  However, doing complicated
host work with the WASM execution suspended could still be helpful to
do deal problems (like timeouts) in the host work.
