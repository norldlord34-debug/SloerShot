// HTTP end-to-end test for the share endpoints. Run: node backend/scripts/test-share-http.js
process.env.SLOERSHOT_NO_LISTEN = "1";
import assert from "node:assert/strict";

const { app } = await import("../src/server.js");
const server = app.listen(0);
await new Promise((r) => server.once("listening", r));
const base = `http://127.0.0.1:${server.address().port}`;

async function post(path, body) {
 const res = await fetch(base + path, {
 method: "POST",
 headers: { "content-type": "application/json" },
 body: JSON.stringify(body ?? {}),
 });
 return { status: res.status, json: await res.json().catch(() => null) };
}

let pass = 0;
function check(name, cond) {
 assert.ok(cond, name);
 pass++;
 console.log("ok -", name);
}

const created = await post("/v1/share", { password: "pw", max_views: 1 });
check("create returns id", created.status === 200 && !!created.json.id);
const id = created.json.id;

const wrong = await post(`/v1/share/${id}`, { password: "nope" });
check("wrong password -> 401", wrong.status === 401);

const right = await post(`/v1/share/${id}`, { password: "pw" });
check("right password -> 200", right.status === 200);

const exhausted = await post(`/v1/share/${id}`, { password: "pw" });
check("self-destruct after max_views -> 410", exhausted.status === 410);

const missing = await post("/v1/share/does-not-exist", {});
check("missing -> 404", missing.status === 404);

const health = await fetch(base + "/health").then((r) => r.json());
check("health ok", health.status === "ok");

const open = await post("/v1/share", {});
check("create open link returns id and /s url", open.status === 200 && !!open.json.id && open.json.url === "/s/" + open.json.id);
const v1 = await post("/v1/share/" + open.json.id, {});
check("open link first view -> 200 and views=1", v1.status === 200 && v1.json.views === 1);
const v2 = await post("/v1/share/" + open.json.id, {});
check("open link second view -> 200 and views=2", v2.status === 200 && v2.json.views === 2);
const nowSec = Math.floor(Date.now() / 1000);
const exp = await post("/v1/share", { expires_at: nowSec - 100 });
const expView = await post("/v1/share/" + exp.json.id, {});
check("expired-by-time link -> 410", expView.status === 410 && expView.json.error === "expired");
const meta = await post("/v1/share", { max_views: 5, expires_at: nowSec + 1000 });
check("create echoes max_views and expires_at", meta.json.max_views === 5 && meta.json.expires_at === nowSec + 1000);
const pk = await fetch(base + "/v1/public-key").then((r) => r.json());
check("public-key is ed25519 hex (64 chars)", pk.alg === "ed25519" && typeof pk.public_key_hex === "string" && pk.public_key_hex.length === 64);
const noSubj = await post("/v1/entitlement", {});
check("entitlement without subject -> 400", noSubj.status === 400);
const ent = await post("/v1/entitlement", { subject: "user@example.com", plan: "Pro", days: 7 });
check("entitlement with subject -> token and claims", ent.status === 200 && typeof ent.json.token === "string" && ent.json.claims.subject === "user@example.com");
server.close();
console.log(`\nALL OK: ${pass} share HTTP e2e checks passed`);
