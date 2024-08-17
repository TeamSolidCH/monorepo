import { describe, expect, it } from "bun:test";

describe("GetIndexTest", () => {
	it('Should return "Hello Hono!"', async () => {
		expect("Hello Hono!").toBe("Hello Hono!");
	});
});
