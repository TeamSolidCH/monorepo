import { describe, expect, it } from "bun:test";
import app from ".";

describe("Default routes", () => {
	it('Should return "Hello Hono!"', async () => {
		const req = new Request("http://localhost/");
		const res = await app.fetch(req);
		expect(await res.text()).toBe("Hello Hono!");
	});
});
