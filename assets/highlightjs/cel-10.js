/*!
 * SPDX-License-Identifier: Apache-2.0
 *
 * Highlight.js 10.1.1 — CEL language definition (repo-authored).
 *
 * Used for Protovalidate message-level rules in ```cel fences. Compatible with
 * mdBook's bundled Highlight.js 10.1.1.
 */
(function () {
  "use strict";
  hljs.registerLanguage("cel", function (hljs) {
    var KEYWORDS =
      "true false null in has this size type dyn uint int double string bytes " +
      "duration timestamp exists all map filter id message expression";
    return {
      name: "Common Expression Language",
      aliases: ["google-cel"],
      keywords: {
        keyword: KEYWORDS,
        literal: "true false null",
      },
      contains: [
        hljs.QUOTE_STRING_MODE,
        hljs.NUMBER_MODE,
        hljs.C_LINE_COMMENT_MODE,
        hljs.C_BLOCK_COMMENT_MODE,
        {
          className: "title",
          begin: /^(id|message|expression)\s*:/,
          relevance: 10,
        },
        {
          className: "function",
          begin: /\b[a-zA-Z_][\w]*\s*\(/,
          relevance: 0,
        },
      ],
    };
  });
  hljs.registerAliases(["google-cel"], { languageName: "cel" });
})();
