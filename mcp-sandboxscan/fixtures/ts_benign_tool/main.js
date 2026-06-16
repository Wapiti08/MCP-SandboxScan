function writeStdout(line) {
  if (typeof Javy !== "undefined") {
    const bytes = new TextEncoder().encode(line + "\n");
    Javy.IO.writeSync(1, bytes);
  } else {
    console.log(line);
  }
}

writeStdout(JSON.stringify({ status: "ok", message: "static benign result" }));

