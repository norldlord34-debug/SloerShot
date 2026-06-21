// Behavioral tests for the share-link logic; run with: node backend/scripts/test-share.js
import assert from "node:assert/strict";
import { createLink, getLink, tryView, hashPassword, _resetForTests } from "../src/share.js";

let pass = 0;
function check(name, fn) {
 _resetForTests();
 fn();
 pass++;
 console.log("ok -", name);
}

check("password hash is sha256 hex (64 chars) and not plaintext", () => {
 const h = hashPassword("hunter2");
 assert.equal(h.length, 64);
 assert.notEqual(h, "hunter2");
 assert.equal(h, hashPassword("hunter2"));
});

check("open link allows a view and counts it", () => {
 const l = createLink({ id: "a", createdAt: 1000 });
 assert.equal(tryView("a", 2000, null).ok, true);
 assert.equal(l.views, 1);
});

check("expired link is blocked", () => {
 createLink({ id: "b", createdAt: 1000, expiresAt: 1500 });
 assert.equal(tryView("b", 2000, null).reason, "expired");
});

check("password is required and checked", () => {
 createLink({ id: "c", createdAt: 1000, password: "s3cret" });
 assert.equal(tryView("c", 1100, null).reason, "bad_password");
 assert.equal(tryView("c", 1100, "nope").reason, "bad_password");
 assert.equal(tryView("c", 1100, "s3cret").ok, true);
});

check("self-destructs after max views", () => {
 createLink({ id: "d", createdAt: 1000, maxViews: 2 });
 assert.equal(tryView("d", 1100, null).ok, true);
 assert.equal(tryView("d", 1100, null).ok, true);
 assert.equal(tryView("d", 1100, null).reason, "exhausted");
});

check("missing link returns not_found", () => {
 assert.equal(tryView("missing", 1, null).reason, "not_found");
});

check("expiry takes precedence over a wrong password", () => {
 createLink({ id: "e", createdAt: 1000, expiresAt: 1500, password: "p" });
 assert.equal(tryView("e", 2000, "wrong").reason, "expired");
});
check("exhaustion takes precedence over a wrong password", () => {
 createLink({ id: "f", createdAt: 1000, maxViews: 1, password: "p" });
 assert.equal(tryView("f", 1100, "p").ok, true);
 assert.equal(tryView("f", 1100, "wrong").reason, "exhausted");
});
check("expiry boundary: now equal to expires_at is expired", () => {
 createLink({ id: "g", createdAt: 1000, expiresAt: 1500 });
 assert.equal(tryView("g", 1500, null).reason, "expired");
 assert.equal(getLink("g").views, 0);
});
check("unconstrained link allows many views", () => {
 createLink({ id: "h", createdAt: 1000 });
 assert.equal(tryView("h", 1100, null).ok, true);
 assert.equal(tryView("h", 1100, null).ok, true);
 assert.equal(tryView("h", 1100, null).ok, true);
 assert.equal(getLink("h").views, 3);
});
check("empty-string password leaves the link open", () => {
 const l = createLink({ id: "i", createdAt: 1000, password: "" });
 assert.equal(l.password_hash, null);
 assert.equal(tryView("i", 1100, null).ok, true);
});
check("numeric password is coerced to a string consistently", () => {
 createLink({ id: "j", createdAt: 1000, password: 1234 });
 assert.equal(tryView("j", 1100, "12").reason, "bad_password");
 assert.equal(tryView("j", 1100, "1234").ok, true);
});
check("auto-generated ids are unique and retrievable", () => {
 const a = createLink({ createdAt: 1 });
 const b = createLink({ createdAt: 1 });
 assert.notEqual(a.id, b.id);
 assert.equal(getLink(a.id), a);
});
console.log(`\nALL OK: ${pass} share tests passed`);
