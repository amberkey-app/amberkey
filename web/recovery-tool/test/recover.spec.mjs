// Loads dist/recover.html from file:// with network disabled, enters test
// shares and the fixture bundle, asserts the packet renders. Seedplan §11.
import { test, expect } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..", "..");
const toolPath = join(here, "..", "dist", "recover.html");

let fixtureDir;
test.beforeAll(() => {
  fixtureDir = mkdtempSync(join(tmpdir(), "amberkey-fixture-"));
  execFileSync("cargo", ["run", "-j2", "-q", "-p", "amberkey-core", "--example", "make_fixture", "--", fixtureDir], {
    cwd: repo,
    stdio: "inherit",
  });
});

test("recovers a packet fully offline from file://", async ({ page }) => {
  const requests = [];
  page.on("request", (r) => {
    if (!r.url().startsWith("file://")) requests.push(r.url());
  });

  await page.goto("file://" + toolPath);
  await expect(page.locator("h1")).toHaveText("AmberKey Recovery");

  // Quorum: spouse (G1) + both kids (G2) = 2 of 2 groups. Leave out G3.
  const shares = readFileSync(join(fixtureDir, "shares.txt"), "utf8").trim().split("\n");
  await page.fill("#share-input", shares.slice(0, 3).join("\n"));
  await page.click("#add-shares");
  await expect(page.locator("#progress")).toContainText("You have enough cards.");

  await page.setInputFiles("#bundle-input", join(fixtureDir, "bundle.age"));
  await page.click("#recover");

  await expect(page.locator("#packet-title")).toContainText("Alex Fixture");
  await expect(page.locator("#view")).toContainText("Executor checklist");
  await expect(page.locator("#view")).toContainText("Do not cancel the phone number");

  // Account card renders
  await page.click('nav#files a:text("packet/cards/google.json")');
  await expect(page.locator("#view")).toContainText("Inactive Account Manager");

  expect(requests, "tool must make zero network requests").toEqual([]);
});

test("rejects below-threshold shares and reports progress", async ({ page }) => {
  await page.goto("file://" + toolPath);
  const shares = readFileSync(join(fixtureDir, "shares.txt"), "utf8").trim().split("\n");
  // Spouse card + one kid: group 2 incomplete.
  await page.fill("#share-input", shares[0] + "\n" + shares[1]);
  await page.click("#add-shares");
  await expect(page.locator("#progress")).toContainText("1 of 2 groups complete");

  await page.setInputFiles("#bundle-input", join(fixtureDir, "bundle.age"));
  await page.click("#recover");
  await expect(page.locator("#recover-error")).toBeVisible();
  await expect(page.locator("#packet")).toBeHidden();
});

test("rejects a mistyped word by checksum", async ({ page }) => {
  await page.goto("file://" + toolPath);
  const shares = readFileSync(join(fixtureDir, "shares.txt"), "utf8").trim().split("\n");
  const words = shares[0].split(" ");
  words[5] = words[5] === "academic" ? "acid" : "academic";
  await page.fill("#share-input", words.join(" "));
  await page.click("#add-shares");
  await expect(page.locator("#share-list")).toContainText("Not accepted");
});
