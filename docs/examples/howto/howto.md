**example:**

```sh
carpenter howto
```

Result (one envelope on stdout):
```json
{"status":"ok","message":"manual printed","data":{"howto":"# carpenter — howto\n…"}}
```

The manual embeds a worked example per command, sourced from `docs/examples/`. Never hand-edit `src/howto.gen.md` — run `cargo xtask gen-howto`.
