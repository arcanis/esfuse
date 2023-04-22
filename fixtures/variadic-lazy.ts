export async function foo() {
  return await import(`./package-exports/export-cjs-${`constants`}.js`);
}
