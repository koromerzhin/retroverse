import { test, expect } from "@playwright/test";

test("home page loads", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle(/RETROVERSE/i);
});

test("sidebar contains navigation", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("Bibliothèque")).toBeVisible();
});
