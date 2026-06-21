// Unit + HTTP e2e tests for real login credential checks and Stripe event mapping.
process.env.SLOERSHOT_NO_LISTEN = "1";
import assert from "node:assert/strict";
import { hashPassword } from "../src/share.js";
import { verifyCredentials } from "../src/auth.js";
import { mapSubscriptionEvent } from "../src/stripe.js";
let pass = 0;
function check(name, cond) {
 assert.ok(cond, name);
 pass++;
 console.log("ok -", name);
}
const users = { "alice@x.com": hashPassword("secret123") };
check("correct password verifies", verifyCredentials("alice@x.com", "secret123", users) === true);
check("wrong password rejected", verifyCredentials("alice@x.com", "nope", users) === false);
check("unknown user rejected", verifyCredentials("bob@x.com", "secret123", users) === false);
check("missing fields rejected", verifyCredentials("", "", users) === false);
const issue = mapSubscriptionEvent({ type: "customer.subscription.created", data: { object: { customer: "cus_1", status: "active", current_period_end: 2000, items: { data: [{ price: { nickname: "Ultra" } }] } } } }, 1000);
check("active subscription -> issue Ultra", issue.action === "issue" && issue.subject === "cus_1" && issue.plan === "Ultra" && issue.expiresAt === 2000);
const dflt = mapSubscriptionEvent({ type: "customer.subscription.updated", data: { object: { customer: "cus_2", status: "active" } } }, 1000);
check("active with no plan -> defaults to Pro", dflt.action === "issue" && dflt.plan === "Pro");
const revoke = mapSubscriptionEvent({ type: "customer.subscription.deleted", data: { object: { customer: "cus_3" } } }, 1000);
check("deleted subscription -> revoke", revoke.action === "revoke" && revoke.subject === "cus_3");
const ignore = mapSubscriptionEvent({ type: "invoice.paid", data: { object: {} } }, 1000);
check("unrelated event -> ignore", ignore.action === "ignore");
process.env.SLOERSHOT_USERS_JSON = JSON.stringify({ "alice@x.com": hashPassword("secret123") });
const { app } = await import("../src/server.js");
const server = app.listen(0);
await new Promise((r) => server.once("listening", r));
const base = `http://127.0.0.1:${server.address().port}`;
async function post(path, body, raw) {
 const res = await fetch(base + path, { method: "POST", headers: { "content-type": "application/json" }, body: raw ? body : JSON.stringify(body ?? {}) });
 return { status: res.status, json: await res.json().catch(() => null) };
}
const noPw = await post("/v1/auth/login", { email: "alice@x.com" });
check("login without password -> 400", noPw.status === 400);
const wrong = await post("/v1/auth/login", { email: "alice@x.com", password: "bad" });
check("login wrong password -> 401", wrong.status === 401);
const okLogin = await post("/v1/auth/login", { email: "alice@x.com", password: "secret123" });
check("login correct -> 200 + session", okLogin.status === 200 && !!okLogin.json.session);
const subEvent = JSON.stringify({ type: "customer.subscription.created", data: { object: { customer: "cus_http", status: "active", current_period_end: 99999999999, items: { data: [{ price: { nickname: "Pro" } }] } } } });
const hook = await post("/v1/stripe/webhook", subEvent, true);
check("webhook active -> 200 issue + token", hook.status === 200 && hook.json.action === "issue" && typeof hook.json.token === "string");
const delEvent = JSON.stringify({ type: "customer.subscription.deleted", data: { object: { customer: "cus_http" } } });
const hookDel = await post("/v1/stripe/webhook", delEvent, true);
check("webhook deleted -> 200 revoke", hookDel.status === 200 && hookDel.json.action === "revoke");
server.close();
console.log(`\nALL OK: ${pass} auth/stripe checks passed`);
