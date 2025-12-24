## Run

```bash
# JSON output
bukvar -i examples/input -o examples/output/json -f json --verbose

# binary
bukvar -i examples/input -o examples/output/dast -f dast --verbose
```

Note: uses `--verbose` not `-v` (that's version). No `-r` flag needed, recursive is on by default.
