/*!
 * SPDX-License-Identifier: BSD-3-Clause
 *
 * Highlight.js 10.1.1 — Protocol Buffers language definition.
 * Upstream: https://github.com/highlightjs/highlight.js/blob/10.1.1/src/languages/protobuf.js
 * Author (upstream): Dan Tao <dan@dtao.org>
 *
 * Local modifications (protoc-gen-mdbook):
 * - Wrapped as hljs.registerLanguage IIFE for mdBook's bundled Highlight.js 10.1.1
 * - Extended keywords: syntax, edition, extend, reserved, map, weak, public
 */
(function () {
  "use strict";
  hljs.registerLanguage("protobuf", function (hljs) {
    return {
      name: "Protocol Buffers",
      aliases: ["proto"],
      keywords: {
        keyword:
          "package import option optional required repeated group oneof syntax edition extend reserved map weak public",
        built_in:
          "double float int32 int64 uint32 uint64 sint32 sint64 " +
          "fixed32 fixed64 sfixed32 sfixed64 bool string bytes",
        literal: "true false",
      },
      contains: [
        hljs.QUOTE_STRING_MODE,
        hljs.NUMBER_MODE,
        hljs.C_LINE_COMMENT_MODE,
        hljs.C_BLOCK_COMMENT_MODE,
        {
          className: "class",
          beginKeywords: "message enum service",
          end: /\{/,
          illegal: /\n/,
          contains: [
            hljs.inherit(hljs.TITLE_MODE, {
              starts: { endsWithParent: true, excludeEnd: true },
            }),
          ],
        },
        {
          className: "function",
          beginKeywords: "rpc",
          end: /[{;]/,
          excludeEnd: true,
          keywords: "rpc returns",
        },
        {
          begin: /^\s*[A-Z_]+/,
          end: /\s*=/,
          excludeEnd: true,
        },
      ],
    };
  });
  hljs.registerAliases(["proto"], { languageName: "protobuf" });
})();
