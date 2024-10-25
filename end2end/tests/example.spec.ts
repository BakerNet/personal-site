import { test, expect } from "@playwright/test";

test("homepage has title and heading text", async ({ page }) => {
  await page.goto("http://localhost:3000/");
});
