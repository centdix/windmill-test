const p = {
  name: "windmill-relative-resolver",
  async setup(build) {
    const { writeFileSync, readFileSync, mkdirSync } = await import("fs");
    const { dirname, resolve } = await import("node:path");

    const base_internal_url = "BASE_INTERNAL_URL".replace(
      "localhost",
      "127.0.0.1"
    );

    const w_id = "W_ID";
    const current_path = "CURRENT_PATH";
    const token = "TOKEN";

    const cdir = resolve("./");
    const cdirNoPrivate = cdir.replace(/^\/private/, ""); // for macos
    const filterResolve = new RegExp(
      `^(?!\\.\/main\\.ts)(?!${cdir}\/main\\.ts)(?!(?:/private)?${cdirNoPrivate}\/wrapper\\.mjs).*\\.ts$`
    );

    let cdirNodeModules = `${cdir}/node_modules/`;

    const filterLoad = new RegExp(`^${cdir}\/main\\.ts$`);
    const transpiler = new Bun.Transpiler({
      loader: "ts",
    });

    function replaceRelativeImports(code) {
      const imports = transpiler.scanImports(code);
      for (const imp of imports) {
        if (imp.kind == "import-statement") {
          if (imp.path.startsWith(".") && !imp.path.endsWith(".ts")) {
            code = code.replaceAll(imp.path, imp.path + ".ts");
          }
        }
      }
      return {
        contents: code,
      };
    }

    build.onLoad({ filter: filterLoad }, async (args) => {
      const code = readFileSync(args.path, "utf8");
      return replaceRelativeImports(code);
    });

    build.onLoad({ filter: /.*\.url$/ }, async (args) => {
      const url = readFileSync(args.path, "utf8");
      const req = await fetch(url, {
        method: "GET",
        headers: {
          Authorization: "Bearer " + token,
        },
      });
      if (!req.ok) {
        throw new Error(
          `Failed to find relative import at ${url}`,
          req.statusText
        );
      }
      const contents = await req.text();
      return {
        contents: replaceRelativeImports(contents).contents,
        loader: "tsx",
      };
    });

    build.onResolve({ filter: filterResolve }, (args) => {
      if (args.importer?.startsWith(cdirNodeModules)) {
        return undefined;
      }

      // Handle absolute HTTP/HTTPS URLs
      if (args.path.startsWith("http://") || args.path.startsWith("https://")) {
        return {
          path: args.path,
          external: true, // Tell Bun to fetch this URL itself
        };
      }

      // Existing logic for local imports:
      const file_path =
        args.importer == "./main.ts" || args.importer == resolve("./main.ts")
          ? current_path
          : args.importer.replace(cdir + "/", ""); // This is the workspace path of the importer

      const isRelative = !args.path.startsWith("/"); // True for "./foo", "../foo", "foo" (relative to importer). False for "/foo" (relative to workspace root).

      let endExt = args.path.endsWith(".ts") ? "" : ".ts";
      // For local paths starting with "/", remove the leading slash for the backend URL segment.
      const localAbsolutePathSegment = args.path.startsWith("/") ? args.path.substring(1) : args.path;

      const url = isRelative
        ? `${base_internal_url}/api/w/${w_id}/scripts/raw_unpinned/p/${file_path}/../${args.path}${endExt}`
        : `${base_internal_url}/api/w/${w_id}/scripts/raw_unpinned/p/${localAbsolutePathSegment}${endExt}`;
      
      // The `file` variable is the path to the .url meta-file created in the job directory.
      // It should be resolved relative to `cdir` (the job directory).
      // For "isRelative = false" cases (args.path starts with "/"), use localAbsolutePathSegment.
      const file = isRelative
        ? resolve(cdir, dirname(file_path), args.path + ".url") // Path relative to importer's directory within workspace structure mirrored in cdir
        : resolve(cdir, localAbsolutePathSegment + ".url"); // Path relative to cdir root, mirroring workspace root

      mkdirSync(dirname(file), { recursive: true });
      writeFileSync(file, url);
      return {
        path: file,
      };
    });
  },
};
