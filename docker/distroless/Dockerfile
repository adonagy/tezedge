FROM gcr.io/distroless/cc-debian10 as light-node
COPY --from=builder /tmp/tezedge /

# Copy binaries
COPY --from=builder /home/appuser/tezedge/target/release/light-node /
COPY --from=builder /home/appuser/tezedge/target/release/protocol-runner /
COPY --from=builder /home/appuser/tezedge/target/release/sandbox /
COPY --from=builder /home/appuser/tezedge/sandbox/artifacts/tezos-client /
COPY --from=builder /home/appuser/tezedge/light_node/etc/tezedge_sandbox/sandbox-patch-context.json /

COPY ./tezedge.config /

# Copy shared libraries
COPY --from=builder /home/appuser/tezedge/tezos/interop/lib_tezos/artifacts/libtezos.so /
COPY --from=builder /usr/lib/x86_64-linux-gnu/libsodium.so.23 /usr/lib/x86_64-linux-gnu/libsodium.so.23
COPY --from=builder /usr/lib/x86_64-linux-gnu/libev.so.4 /usr/lib/x86_64-linux-gnu/libev.so.4
COPY --from=builder /usr/lib/x86_64-linux-gnu/libgmp.so.10 /usr/lib/x86_64-linux-gnu/libgmp.so.10

ENV LD_LIBRARY_PATH="/"
CMD [ "/light-node", "--config-file", "/tezedge.config"]

FROM light-node as sandbox

ENV LD_LIBRARY_PATH="/"
# Add aditional libs used by the sandbox module
# Libs required by tezos-client
COPY --from=builder /usr/lib/x86_64-linux-gnu/libhidapi-libusb.so.0 /usr/lib/x86_64-linux-gnu/libhidapi-libusb.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libusb-1.0.so.0 /lib/x86_64-linux-gnu/libusb-1.0.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libudev.so.1 /lib/x86_64-linux-gnu/libudev.so.1

CMD [ "/sandbox", "--log-level", "debug", "--sandbox-rpc-port", "3030", "--light-node-path", "/light-node", "--tezos-client-path", "/tezos-client"]
