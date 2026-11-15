import { test, expect } from "@playwright/test";

const pages = [
  { path: "/", text: "Accueil" },
  { path: "/library", text: "Bibliothèque" },
  { path: "/platforms", text: "Plateformes" },
  { path: "/sources", text: "Sources" },
  { path: "/settings", text: "Paramètres" },
  { path: "/stats", text: "Statistiques" },
  { path: "/collections", text: "Collections" },
];

for (const p of pages) {
  test(`${p.path} renders without crash`, async ({ page }) => {
    await page.goto(p.path);
    await expect(page).toHaveTitle(/RETROVERSE/i);
    await expect(page.getByText(p.text).first()).toBeVisible({ timeout: 10000 });
  });
}

test("sidebar navigation works", async ({ page }) => {
  await page.goto("/");
  await page.getByText("Bibliothèque").first().click();
  await expect(page).toHaveURL(/library/);
});

test("keyboard shortcut ? opens help", async ({ page }) => {
  await page.goto("/");
  await page.locator("body").click();
  await page.keyboard.press("Shift+/");
  await expect(page.getByText("Raccourcis clavier")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByText("Raccourcis clavier")).not.toBeVisible();
});
