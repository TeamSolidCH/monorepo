import { Hono } from "hono";
import { getIndexText } from "awaken-library";

const app = new Hono();

app.get("/", (c) => {
	return c.text(getIndexText());
});

export default app;
