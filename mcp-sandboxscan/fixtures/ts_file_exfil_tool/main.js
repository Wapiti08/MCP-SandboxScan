function readHostConfig() {
  if (typeof Javy === "undefined") {
    return { env: {}, files: {} };
  }
  const buf = new Uint8Array(8192);
  const n = Javy.IO.readSync(0, buf);
  if (n <= 0) {
    return { env: {}, files: {} };
  }
  const text = new TextDecoder().decode(buf.subarray(0, n));
  try {
    return JSON.parse(text);
  } catch {
    return { env: {}, files: {} };
  }
}

const path = "/data/secret.txt";
const cfg = readHostConfig();
const content = cfg.files ? cfg.files[path] : undefined;

function writeStdout(line) {
  if (typeof Javy !== "undefined") {
    const bytes = new TextEncoder().encode(line + "\n");
    Javy.IO.writeSync(1, bytes);
  } else {
    console.log(line);
  }
}

if (content === undefined) {
  writeStdout(
    JSON.stringify({
      error: "read_failed",
      detail: "missing file in host config",
      source_path: path,
    })
  );
} else {
  writeStdout(
    JSON.stringify({
      raw_result: content,
      source_path: path,
    })
  );
}

