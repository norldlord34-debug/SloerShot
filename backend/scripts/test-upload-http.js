// HTTP end-to-end smoke test for the image upload engine path used by the desktop apps.
// Mirrors what the Windows/macOS uploader does against a Custom Uploader / SloerShot Cloud
// destination: binary POST to /v1/upload -> parse {url} from JSON -> fetch it back.
// Run: node backend/scripts/test-upload-http.js
process.env.SLOERSHOT_NO_LISTEN = "1";
import assert from "node:assert/strict";

const { app } = await import("../src/server.js");
const server = app.listen(0);
await new Promise((r) => server.once("listening", r));
const base = `http://127.0.0.1:${server.address().port}`;

let pass = 0;
function check(name, cond) {
 assert.ok(cond, name);
 pass++;
 console.log("ok -", name);
}

// A real 1x1 PNG (the smallest valid image the engine could ship).
const png = Buffer.from(
 "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
 "base64"
);

// 1) Binary upload -> 200 with { id, url } (matches Custom Uploader "Binary" body + {json:url}).
const up = await fetch(base + "/v1/upload", {
 method: "POST",
 headers: { "content-type": "image/png" },
 body: png,
});
const upJson = await up.json().catch(() => null);
check("binary upload -> 200", up.status === 200);
check("upload returns an id ending in .png", !!upJson && typeof upJson.id === "string" && upJson.id.endsWith(".png"));
check("upload returns an absolute http url", !!upJson && typeof upJson.url === "string" && upJson.url.startsWith("http"));

// 2) The returned url must serve the exact bytes back with an image content-type.
const path = new URL(upJson.url).pathname;
const got = await fetch(base + path);
const gotBuf = Buffer.from(await got.arrayBuffer());
check("hosted file -> 200", got.status === 200);
check("hosted file content-type is image/png", (got.headers.get("content-type") || "").includes("image/png"));
check("hosted bytes round-trip byte-for-byte", gotBuf.equals(png));

// 3) A jpeg content-type should be honored in the stored extension.
const upJpg = await fetch(base + "/v1/upload", {
 method: "POST",
 headers: { "content-type": "image/jpeg" },
 body: png,
});
const upJpgJson = await upJpg.json().catch(() => null);
check("jpeg upload stored with .jpg extension", upJpg.status === 200 && !!upJpgJson && upJpgJson.id.endsWith(".jpg"));

// 4) Empty body must be rejected (the app never uploads a zero-byte capture).
const empty = await fetch(base + "/v1/upload", { method: "POST", headers: { "content-type": "image/png" } });
check("empty upload body -> 400", empty.status === 400);

// 5) Missing hosted file -> 404 (deletion / bad id path).
const missing = await fetch(base + "/f/does-not-exist.png");
check("missing hosted file -> 404", missing.status === 404);

server.close();
console.log(`\nALL OK: ${pass} upload-engine HTTP e2e checks passed`);
