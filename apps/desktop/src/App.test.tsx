import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("M2 desktop shell", () => {
  it("renders an honest not-yet-implemented state", () => {
    const markup = renderToStaticMarkup(<App />);

    expect(markup).toContain("Voxora");
    expect(markup).toContain("M2 workspace skeleton");
    expect(markup).toContain(
      "Dictation features will be added in a later milestone.",
    );
    expect(markup).toContain("No session is running.");
  });
});
