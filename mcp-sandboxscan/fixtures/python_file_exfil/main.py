import json

path = "/data/secret.txt"
try:
    with open(path) as f:
        s = f.read()
    print(json.dumps({"raw_result": s, "source_path": path}))
except OSError as e:
    print(json.dumps({"error": "read_failed", "detail": str(e), "source_path": path}))
