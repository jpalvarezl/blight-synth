export {};

const requestedBase = Bun.argv[2] ?? "./";
const index = Bun.file("dist/index.html");

if (!(await index.exists())) {
  throw new Error("dist/index.html is missing; run the production build first");
}

const html = await index.text();
const references = [...html.matchAll(/(?:src|href)="([^"]+)"/g)].map(
  (match) => match[1],
);
const assetReferences = references.filter((path) => path?.includes("assets/"));

if (assetReferences.length === 0) {
  throw new Error("dist/index.html does not reference compiled assets");
}

if (/\/src\/|\/@vite|localhost:\d+/.test(html)) {
  throw new Error(
    "dist/index.html still contains a development-server reference",
  );
}

const expectedPrefix =
  requestedBase === "./" ? "./assets/" : `${requestedBase}assets/`;
for (const reference of assetReferences) {
  if (!reference?.startsWith(expectedPrefix)) {
    throw new Error(
      `asset reference ${reference} does not use expected base ${expectedPrefix}`,
    );
  }

  const filename = reference.slice(reference.indexOf("assets/"));
  if (!(await Bun.file(`dist/${filename}`).exists())) {
    throw new Error(`compiled asset ${filename} is missing`);
  }
}

console.log(
  `verified ${assetReferences.length} static asset references with base ${requestedBase}`,
);
