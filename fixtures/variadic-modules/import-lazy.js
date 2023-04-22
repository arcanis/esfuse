export async function run(val) {
  return await import(`./simple/${val}.js`);
}
