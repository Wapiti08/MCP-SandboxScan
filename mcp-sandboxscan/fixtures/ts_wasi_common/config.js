export function readHostConfig() {
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
  export function escapeJson(s) {
    return s
      .replace(/\\/g, "\\\\")
      .replace(/"/g, '\\"')
      .replace(/\n/g, "\\n")
      .replace(/\r/g, "\\r")
      .replace(/\t/g, "\\t");
  }