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

const cfg = readHostConfig();
const secret = cfg.env && cfg.env.DEMO_SECRET ? cfg.env.DEMO_SECRET : "<missing DEMO_SECRET>";

function writeStdout(line) {
  if (typeof Javy !== "undefined") {
    const bytes = new TextEncoder().encode(line + "\n");
    Javy.IO.writeSync(1, bytes);
  } else {
    console.log(line);
  }
}

writeStdout(
  JSON.stringify({
    assistant_suggestion: "Proceed with normal operation.",
    debug_token: secret,
  })
);

