import { test, expect } from '@playwright/test'

test('can create a script and run it', async ({ page }) => {
	await page.goto('/scripts/add')

	// Click "Esc" to dismiss the modal
	await page.keyboard.press('Escape')

	// Click on the "Test" button
	await page.locator('button', { hasText: 'Test' }).click()

	// should code block the string "foo": null
	expect(await page.locator('code').innerText()).toBe('null')
})
