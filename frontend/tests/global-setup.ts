import { chromium } from '@playwright/test'

async function globalSetup() {
	const browser = await chromium.launch()
	// baseURL is set in the global config, but it doesn't affect the globalSetup script.
	const page = await browser.newPage({ baseURL: process.env.BASE_URL || 'http://localhost:3000' })
	await page.goto('/user/login', { waitUntil: 'networkidle' })

	if (await page.locator('#email').isHidden()) {
		await page
			.locator('button', {
				hasText: 'Log in without third-party'
			})
			.click()
	}

	const email = page.locator('input[type="email"]')
	await email.fill('user@windmill.dev')

	const password = page.locator('input[type="password"]')
	await password.fill('changeme')

	// click on button with test "Sign in"

	await page.locator('button', { hasText: 'Sign in' }).click()

	await page.waitForResponse('/api/auth/login')
	await page.context().storageState({ path: 'storageState.json' })
	await browser.close()
}

export default globalSetup
