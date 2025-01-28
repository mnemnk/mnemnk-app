module.exports = {
  bumpFiles: [
    {
      filename: "package.json",
      type: "json",
    },
    {
      filename: "package-lock.json",
      type: "json",
    },
    {
      filename: "src-tauri/tauri.conf.json",
      type: "json",
    },
    {
      filename: "src-tauri/Cargo.toml",
      updater: {
        readVersion: (contents) => {
          const match = contents.match(/^version\s*=\s*"([^"]+)"/m);
          return match ? match[1] : null;
        },
        writeVersion: (contents, version) => {
          return contents.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
        },
      },
    },
    {
      filename: "src-tauri/Cargo.lock",
      updater: {
        readVersion: (contents) => {
          const regex = /\[\[package\]\]\s*name\s*=\s*"mnemnk-app"\s*version\s*=\s*"([^"]+)"/m;
          const match = contents.match(regex);
          return match ? match[1] : null;
        },
        writeVersion: (contents, version) => {
          const result = contents.replace(
            /(\[\[package\]\]\s*name\s*=\s*"mnemnk-app"\s*version\s*=\s*")[^"]+(")/,
            `$1${version}$2`,
          );
          return result;
        },
      },
    },
  ],
};
