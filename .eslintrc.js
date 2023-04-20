const sections = [
  `^@esfuse/`,
  `esfuse`,
];

module.exports = {
  extends: [
    require.resolve(`@yarnpkg/eslint-config`),
    require.resolve(`@yarnpkg/eslint-config/react`),
  ],
  env: {
    node: true,
  },
  rules: {
    [`arca/import-absolutes`]: `error`,
    [`arca/import-quotes`]: `error`,
    [`arca/import-ordering`]: [`error`, {sections, hoistOneliners: true}],
    [`arca/newline-after-import-section`]: [`error`, {sections}],
  },
};
