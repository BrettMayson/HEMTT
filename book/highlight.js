/*!
  Highlight.js v11.10.0 (git: 366a8bd012)
  (c) 2006-2024 Josh Goebel <hello@joshgoebel.com> and other contributors
  License: BSD-3-Clause
 */
var hljs = (function () {
  'use strict';

  /* eslint-disable no-multi-assign */

  function deepFreeze(obj) {
    if (obj instanceof Map) {
      obj.clear =
        obj.delete =
        obj.set =
          function () {
            throw new Error('map is read-only');
          };
    } else if (obj instanceof Set) {
      obj.add =
        obj.clear =
        obj.delete =
          function () {
            throw new Error('set is read-only');
          };
    }

    // Freeze self
    Object.freeze(obj);

    Object.getOwnPropertyNames(obj).forEach((name) => {
      const prop = obj[name];
      const type = typeof prop;

      // Freeze prop if it is an object or function and also not already frozen
      if ((type === 'object' || type === 'function') && !Object.isFrozen(prop)) {
        deepFreeze(prop);
      }
    });

    return obj;
  }

  /** @typedef {import('highlight.js').CallbackResponse} CallbackResponse */
  /** @typedef {import('highlight.js').CompiledMode} CompiledMode */
  /** @implements CallbackResponse */

  class Response {
    /**
     * @param {CompiledMode} mode
     */
    constructor(mode) {
      // eslint-disable-next-line no-undefined
      if (mode.data === undefined) mode.data = {};

      this.data = mode.data;
      this.isMatchIgnored = false;
    }

    ignoreMatch() {
      this.isMatchIgnored = true;
    }
  }

  /**
   * @param {string} value
   * @returns {string}
   */
  function escapeHTML(value) {
    return value
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#x27;');
  }

  /**
   * performs a shallow merge of multiple objects into one
   *
   * @template T
   * @param {T} original
   * @param {Record<string,any>[]} objects
   * @returns {T} a single new object
   */
  function inherit$1(original, ...objects) {
    /** @type Record<string,any> */
    const result = Object.create(null);

    for (const key in original) {
      result[key] = original[key];
    }
    objects.forEach(function(obj) {
      for (const key in obj) {
        result[key] = obj[key];
      }
    });
    return /** @type {T} */ (result);
  }

  /**
   * @typedef {object} Renderer
   * @property {(text: string) => void} addText
   * @property {(node: Node) => void} openNode
   * @property {(node: Node) => void} closeNode
   * @property {() => string} value
   */

  /** @typedef {{scope?: string, language?: string, sublanguage?: boolean}} Node */
  /** @typedef {{walk: (r: Renderer) => void}} Tree */
  /** */

  const SPAN_CLOSE = '</span>';

  /**
   * Determines if a node needs to be wrapped in <span>
   *
   * @param {Node} node */
  const emitsWrappingTags = (node) => {
    // rarely we can have a sublanguage where language is undefined
    // TODO: track down why
    return !!node.scope;
  };

  /**
   *
   * @param {string} name
   * @param {{prefix:string}} options
   */
  const scopeToCSSClass = (name, { prefix }) => {
    // sub-language
    if (name.startsWith("language:")) {
      return name.replace("language:", "language-");
    }
    // tiered scope: comment.line
    if (name.includes(".")) {
      const pieces = name.split(".");
      return [
        `${prefix}${pieces.shift()}`,
        ...(pieces.map((x, i) => `${x}${"_".repeat(i + 1)}`))
      ].join(" ");
    }
    // simple scope
    return `${prefix}${name}`;
  };

  /** @type {Renderer} */
  class HTMLRenderer {
    /**
     * Creates a new HTMLRenderer
     *
     * @param {Tree} parseTree - the parse tree (must support `walk` API)
     * @param {{classPrefix: string}} options
     */
    constructor(parseTree, options) {
      this.buffer = "";
      this.classPrefix = options.classPrefix;
      parseTree.walk(this);
    }

    /**
     * Adds texts to the output stream
     *
     * @param {string} text */
    addText(text) {
      this.buffer += escapeHTML(text);
    }

    /**
     * Adds a node open to the output stream (if needed)
     *
     * @param {Node} node */
    openNode(node) {
      if (!emitsWrappingTags(node)) return;

      const className = scopeToCSSClass(node.scope,
        { prefix: this.classPrefix });
      this.span(className);
    }

    /**
     * Adds a node close to the output stream (if needed)
     *
     * @param {Node} node */
    closeNode(node) {
      if (!emitsWrappingTags(node)) return;

      this.buffer += SPAN_CLOSE;
    }

    /**
     * returns the accumulated buffer
    */
    value() {
      return this.buffer;
    }

    // helpers

    /**
     * Builds a span element
     *
     * @param {string} className */
    span(className) {
      this.buffer += `<span class="${className}">`;
    }
  }

  /** @typedef {{scope?: string, language?: string, children: Node[]} | string} Node */
  /** @typedef {{scope?: string, language?: string, children: Node[]} } DataNode */
  /** @typedef {import('highlight.js').Emitter} Emitter */
  /**  */

  /** @returns {DataNode} */
  const newNode = (opts = {}) => {
    /** @type DataNode */
    const result = { children: [] };
    Object.assign(result, opts);
    return result;
  };

  class TokenTree {
    constructor() {
      /** @type DataNode */
      this.rootNode = newNode();
      this.stack = [this.rootNode];
    }

    get top() {
      return this.stack[this.stack.length - 1];
    }

    get root() { return this.rootNode; }

    /** @param {Node} node */
    add(node) {
      this.top.children.push(node);
    }

    /** @param {string} scope */
    openNode(scope) {
      /** @type Node */
      const node = newNode({ scope });
      this.add(node);
      this.stack.push(node);
    }

    closeNode() {
      if (this.stack.length > 1) {
        return this.stack.pop();
      }
      // eslint-disable-next-line no-undefined
      return undefined;
    }

    closeAllNodes() {
      while (this.closeNode());
    }

    toJSON() {
      return JSON.stringify(this.rootNode, null, 4);
    }

    /**
     * @typedef { import("./html_renderer").Renderer } Renderer
     * @param {Renderer} builder
     */
    walk(builder) {
      // this does not
      return this.constructor._walk(builder, this.rootNode);
      // this works
      // return TokenTree._walk(builder, this.rootNode);
    }

    /**
     * @param {Renderer} builder
     * @param {Node} node
     */
    static _walk(builder, node) {
      if (typeof node === "string") {
        builder.addText(node);
      } else if (node.children) {
        builder.openNode(node);
        node.children.forEach((child) => this._walk(builder, child));
        builder.closeNode(node);
      }
      return builder;
    }

    /**
     * @param {Node} node
     */
    static _collapse(node) {
      if (typeof node === "string") return;
      if (!node.children) return;

      if (node.children.every(el => typeof el === "string")) {
        // node.text = node.children.join("");
        // delete node.children;
        node.children = [node.children.join("")];
      } else {
        node.children.forEach((child) => {
          TokenTree._collapse(child);
        });
      }
    }
  }

  /**
    Currently this is all private API, but this is the minimal API necessary
    that an Emitter must implement to fully support the parser.

    Minimal interface:

    - addText(text)
    - __addSublanguage(emitter, subLanguageName)
    - startScope(scope)
    - endScope()
    - finalize()
    - toHTML()

  */

  /**
   * @implements {Emitter}
   */
  class TokenTreeEmitter extends TokenTree {
    /**
     * @param {*} options
     */
    constructor(options) {
      super();
      this.options = options;
    }

    /**
     * @param {string} text
     */
    addText(text) {
      if (text === "") { return; }

      this.add(text);
    }

    /** @param {string} scope */
    startScope(scope) {
      this.openNode(scope);
    }

    endScope() {
      this.closeNode();
    }

    /**
     * @param {Emitter & {root: DataNode}} emitter
     * @param {string} name
     */
    __addSublanguage(emitter, name) {
      /** @type DataNode */
      const node = emitter.root;
      if (name) node.scope = `language:${name}`;

      this.add(node);
    }

    toHTML() {
      const renderer = new HTMLRenderer(this, this.options);
      return renderer.value();
    }

    finalize() {
      this.closeAllNodes();
      return true;
    }
  }

  /**
   * @param {string} value
   * @returns {RegExp}
   * */

  /**
   * @param {RegExp | string } re
   * @returns {string}
   */
  function source(re) {
    if (!re) return null;
    if (typeof re === "string") return re;

    return re.source;
  }

  /**
   * @param {RegExp | string } re
   * @returns {string}
   */
  function lookahead(re) {
    return concat('(?=', re, ')');
  }

  /**
   * @param {RegExp | string } re
   * @returns {string}
   */
  function anyNumberOfTimes(re) {
    return concat('(?:', re, ')*');
  }

  /**
   * @param {RegExp | string } re
   * @returns {string}
   */
  function optional(re) {
    return concat('(?:', re, ')?');
  }

  /**
   * @param {...(RegExp | string) } args
   * @returns {string}
   */
  function concat(...args) {
    const joined = args.map((x) => source(x)).join("");
    return joined;
  }

  /**
   * @param { Array<string | RegExp | Object> } args
   * @returns {object}
   */
  function stripOptionsFromArgs(args) {
    const opts = args[args.length - 1];

    if (typeof opts === 'object' && opts.constructor === Object) {
      args.splice(args.length - 1, 1);
      return opts;
    } else {
      return {};
    }
  }

  /** @typedef { {capture?: boolean} } RegexEitherOptions */

  /**
   * Any of the passed expresssions may match
   *
   * Creates a huge this | this | that | that match
   * @param {(RegExp | string)[] | [...(RegExp | string)[], RegexEitherOptions]} args
   * @returns {string}
   */
  function either(...args) {
    /** @type { object & {capture?: boolean} }  */
    const opts = stripOptionsFromArgs(args);
    const joined = '('
      + (opts.capture ? "" : "?:")
      + args.map((x) => source(x)).join("|") + ")";
    return joined;
  }

  /**
   * @param {RegExp | string} re
   * @returns {number}
   */
  function countMatchGroups(re) {
    return (new RegExp(re.toString() + '|')).exec('').length - 1;
  }

  /**
   * Does lexeme start with a regular expression match at the beginning
   * @param {RegExp} re
   * @param {string} lexeme
   */
  function startsWith(re, lexeme) {
    const match = re && re.exec(lexeme);
    return match && match.index === 0;
  }

  // BACKREF_RE matches an open parenthesis or backreference. To avoid
  // an incorrect parse, it additionally matches the following:
  // - [...] elements, where the meaning of parentheses and escapes change
  // - other escape sequences, so we do not misparse escape sequences as
  //   interesting elements
  // - non-matching or lookahead parentheses, which do not capture. These
  //   follow the '(' with a '?'.
  const BACKREF_RE = /\[(?:[^\\\]]|\\.)*\]|\(\??|\\([1-9][0-9]*)|\\./;

  // **INTERNAL** Not intended for outside usage
  // join logically computes regexps.join(separator), but fixes the
  // backreferences so they continue to match.
  // it also places each individual regular expression into it's own
  // match group, keeping track of the sequencing of those match groups
  // is currently an exercise for the caller. :-)
  /**
   * @param {(string | RegExp)[]} regexps
   * @param {{joinWith: string}} opts
   * @returns {string}
   */
  function _rewriteBackreferences(regexps, { joinWith }) {
    let numCaptures = 0;

    return regexps.map((regex) => {
      numCaptures += 1;
      const offset = numCaptures;
      let re = source(regex);
      let out = '';

      while (re.length > 0) {
        const match = BACKREF_RE.exec(re);
        if (!match) {
          out += re;
          break;
        }
        out += re.substring(0, match.index);
        re = re.substring(match.index + match[0].length);
        if (match[0][0] === '\\' && match[1]) {
          // Adjust the backreference.
          out += '\\' + String(Number(match[1]) + offset);
        } else {
          out += match[0];
          if (match[0] === '(') {
            numCaptures++;
          }
        }
      }
      return out;
    }).map(re => `(${re})`).join(joinWith);
  }

  /** @typedef {import('highlight.js').Mode} Mode */
  /** @typedef {import('highlight.js').ModeCallback} ModeCallback */

  // Common regexps
  const MATCH_NOTHING_RE = /\b\B/;
  const IDENT_RE = '[a-zA-Z]\\w*';
  const UNDERSCORE_IDENT_RE = '[a-zA-Z_]\\w*';
  const NUMBER_RE = '\\b\\d+(\\.\\d+)?';
  const C_NUMBER_RE = '(-?)(\\b0[xX][a-fA-F0-9]+|(\\b\\d+(\\.\\d*)?|\\.\\d+)([eE][-+]?\\d+)?)'; // 0x..., 0..., decimal, float
  const BINARY_NUMBER_RE = '\\b(0b[01]+)'; // 0b...
  const RE_STARTERS_RE = '!|!=|!==|%|%=|&|&&|&=|\\*|\\*=|\\+|\\+=|,|-|-=|/=|/|:|;|<<|<<=|<=|<|===|==|=|>>>=|>>=|>=|>>>|>>|>|\\?|\\[|\\{|\\(|\\^|\\^=|\\||\\|=|\\|\\||~';

  /**
  * @param { Partial<Mode> & {binary?: string | RegExp} } opts
  */
  const SHEBANG = (opts = {}) => {
    const beginShebang = /^#![ ]*\//;
    if (opts.binary) {
      opts.begin = concat(
        beginShebang,
        /.*\b/,
        opts.binary,
        /\b.*/);
    }
    return inherit$1({
      scope: 'meta',
      begin: beginShebang,
      end: /$/,
      relevance: 0,
      /** @type {ModeCallback} */
      "on:begin": (m, resp) => {
        if (m.index !== 0) resp.ignoreMatch();
      }
    }, opts);
  };

  // Common modes
  const BACKSLASH_ESCAPE = {
    begin: '\\\\[\\s\\S]', relevance: 0
  };
  const APOS_STRING_MODE = {
    scope: 'string',
    begin: '\'',
    end: '\'',
    illegal: '\\n',
    contains: [BACKSLASH_ESCAPE]
  };
  const QUOTE_STRING_MODE = {
    scope: 'string',
    begin: '"',
    end: '"',
    illegal: '\\n',
    contains: [BACKSLASH_ESCAPE]
  };
  const PHRASAL_WORDS_MODE = {
    begin: /\b(a|an|the|are|I'm|isn't|don't|doesn't|won't|but|just|should|pretty|simply|enough|gonna|going|wtf|so|such|will|you|your|they|like|more)\b/
  };
  /**
   * Creates a comment mode
   *
   * @param {string | RegExp} begin
   * @param {string | RegExp} end
   * @param {Mode | {}} [modeOptions]
   * @returns {Partial<Mode>}
   */
  const COMMENT = function(begin, end, modeOptions = {}) {
    const mode = inherit$1(
      {
        scope: 'comment',
        begin,
        end,
        contains: []
      },
      modeOptions
    );
    mode.contains.push({
      scope: 'doctag',
      // hack to avoid the space from being included. the space is necessary to
      // match here to prevent the plain text rule below from gobbling up doctags
      begin: '[ ]*(?=(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):)',
      end: /(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):/,
      excludeBegin: true,
      relevance: 0
    });
    const ENGLISH_WORD = either(
      // list of common 1 and 2 letter words in English
      "I",
      "a",
      "is",
      "so",
      "us",
      "to",
      "at",
      "if",
      "in",
      "it",
      "on",
      // note: this is not an exhaustive list of contractions, just popular ones
      /[A-Za-z]+['](d|ve|re|ll|t|s|n)/, // contractions - can't we'd they're let's, etc
      /[A-Za-z]+[-][a-z]+/, // `no-way`, etc.
      /[A-Za-z][a-z]{2,}/ // allow capitalized words at beginning of sentences
    );
    // looking like plain text, more likely to be a comment
    mode.contains.push(
      {
        // TODO: how to include ", (, ) without breaking grammars that use these for
        // comment delimiters?
        // begin: /[ ]+([()"]?([A-Za-z'-]{3,}|is|a|I|so|us|[tT][oO]|at|if|in|it|on)[.]?[()":]?([.][ ]|[ ]|\))){3}/
        // ---

        // this tries to find sequences of 3 english words in a row (without any
        // "programming" type syntax) this gives us a strong signal that we've
        // TRULY found a comment - vs perhaps scanning with the wrong language.
        // It's possible to find something that LOOKS like the start of the
        // comment - but then if there is no readable text - good chance it is a
        // false match and not a comment.
        //
        // for a visual example please see:
        // https://github.com/highlightjs/highlight.js/issues/2827

        begin: concat(
          /[ ]+/, // necessary to prevent us gobbling up doctags like /* @author Bob Mcgill */
          '(',
          ENGLISH_WORD,
          /[.]?[:]?([.][ ]|[ ])/,
          '){3}') // look for 3 words in a row
      }
    );
    return mode;
  };
  const C_LINE_COMMENT_MODE = COMMENT('//', '$');
  const C_BLOCK_COMMENT_MODE = COMMENT('/\\*', '\\*/');
  const HASH_COMMENT_MODE = COMMENT('#', '$');
  const NUMBER_MODE = {
    scope: 'number',
    begin: NUMBER_RE,
    relevance: 0
  };
  const C_NUMBER_MODE = {
    scope: 'number',
    begin: C_NUMBER_RE,
    relevance: 0
  };
  const BINARY_NUMBER_MODE = {
    scope: 'number',
    begin: BINARY_NUMBER_RE,
    relevance: 0
  };
  const REGEXP_MODE = {
    scope: "regexp",
    begin: /\/(?=[^/\n]*\/)/,
    end: /\/[gimuy]*/,
    contains: [
      BACKSLASH_ESCAPE,
      {
        begin: /\[/,
        end: /\]/,
        relevance: 0,
        contains: [BACKSLASH_ESCAPE]
      }
    ]
  };
  const TITLE_MODE = {
    scope: 'title',
    begin: IDENT_RE,
    relevance: 0
  };
  const UNDERSCORE_TITLE_MODE = {
    scope: 'title',
    begin: UNDERSCORE_IDENT_RE,
    relevance: 0
  };
  const METHOD_GUARD = {
    // excludes method names from keyword processing
    begin: '\\.\\s*' + UNDERSCORE_IDENT_RE,
    relevance: 0
  };

  /**
   * Adds end same as begin mechanics to a mode
   *
   * Your mode must include at least a single () match group as that first match
   * group is what is used for comparison
   * @param {Partial<Mode>} mode
   */
  const END_SAME_AS_BEGIN = function(mode) {
    return Object.assign(mode,
      {
        /** @type {ModeCallback} */
        'on:begin': (m, resp) => { resp.data._beginMatch = m[1]; },
        /** @type {ModeCallback} */
        'on:end': (m, resp) => { if (resp.data._beginMatch !== m[1]) resp.ignoreMatch(); }
      });
  };

  var MODES = /*#__PURE__*/Object.freeze({
    __proto__: null,
    APOS_STRING_MODE: APOS_STRING_MODE,
    BACKSLASH_ESCAPE: BACKSLASH_ESCAPE,
    BINARY_NUMBER_MODE: BINARY_NUMBER_MODE,
    BINARY_NUMBER_RE: BINARY_NUMBER_RE,
    COMMENT: COMMENT,
    C_BLOCK_COMMENT_MODE: C_BLOCK_COMMENT_MODE,
    C_LINE_COMMENT_MODE: C_LINE_COMMENT_MODE,
    C_NUMBER_MODE: C_NUMBER_MODE,
    C_NUMBER_RE: C_NUMBER_RE,
    END_SAME_AS_BEGIN: END_SAME_AS_BEGIN,
    HASH_COMMENT_MODE: HASH_COMMENT_MODE,
    IDENT_RE: IDENT_RE,
    MATCH_NOTHING_RE: MATCH_NOTHING_RE,
    METHOD_GUARD: METHOD_GUARD,
    NUMBER_MODE: NUMBER_MODE,
    NUMBER_RE: NUMBER_RE,
    PHRASAL_WORDS_MODE: PHRASAL_WORDS_MODE,
    QUOTE_STRING_MODE: QUOTE_STRING_MODE,
    REGEXP_MODE: REGEXP_MODE,
    RE_STARTERS_RE: RE_STARTERS_RE,
    SHEBANG: SHEBANG,
    TITLE_MODE: TITLE_MODE,
    UNDERSCORE_IDENT_RE: UNDERSCORE_IDENT_RE,
    UNDERSCORE_TITLE_MODE: UNDERSCORE_TITLE_MODE
  });

  /**
  @typedef {import('highlight.js').CallbackResponse} CallbackResponse
  @typedef {import('highlight.js').CompilerExt} CompilerExt
  */

  // Grammar extensions / plugins
  // See: https://github.com/highlightjs/highlight.js/issues/2833

  // Grammar extensions allow "syntactic sugar" to be added to the grammar modes
  // without requiring any underlying changes to the compiler internals.

  // `compileMatch` being the perfect small example of now allowing a grammar
  // author to write `match` when they desire to match a single expression rather
  // than being forced to use `begin`.  The extension then just moves `match` into
  // `begin` when it runs.  Ie, no features have been added, but we've just made
  // the experience of writing (and reading grammars) a little bit nicer.

  // ------

  // TODO: We need negative look-behind support to do this properly
  /**
   * Skip a match if it has a preceding dot
   *
   * This is used for `beginKeywords` to prevent matching expressions such as
   * `bob.keyword.do()`. The mode compiler automatically wires this up as a
   * special _internal_ 'on:begin' callback for modes with `beginKeywords`
   * @param {RegExpMatchArray} match
   * @param {CallbackResponse} response
   */
  function skipIfHasPrecedingDot(match, response) {
    const before = match.input[match.index - 1];
    if (before === ".") {
      response.ignoreMatch();
    }
  }

  /**
   *
   * @type {CompilerExt}
   */
  function scopeClassName(mode, _parent) {
    // eslint-disable-next-line no-undefined
    if (mode.className !== undefined) {
      mode.scope = mode.className;
      delete mode.className;
    }
  }

  /**
   * `beginKeywords` syntactic sugar
   * @type {CompilerExt}
   */
  function beginKeywords(mode, parent) {
    if (!parent) return;
    if (!mode.beginKeywords) return;

    // for languages with keywords that include non-word characters checking for
    // a word boundary is not sufficient, so instead we check for a word boundary
    // or whitespace - this does no harm in any case since our keyword engine
    // doesn't allow spaces in keywords anyways and we still check for the boundary
    // first
    mode.begin = '\\b(' + mode.beginKeywords.split(' ').join('|') + ')(?!\\.)(?=\\b|\\s)';
    mode.__beforeBegin = skipIfHasPrecedingDot;
    mode.keywords = mode.keywords || mode.beginKeywords;
    delete mode.beginKeywords;

    // prevents double relevance, the keywords themselves provide
    // relevance, the mode doesn't need to double it
    // eslint-disable-next-line no-undefined
    if (mode.relevance === undefined) mode.relevance = 0;
  }

  /**
   * Allow `illegal` to contain an array of illegal values
   * @type {CompilerExt}
   */
  function compileIllegal(mode, _parent) {
    if (!Array.isArray(mode.illegal)) return;

    mode.illegal = either(...mode.illegal);
  }

  /**
   * `match` to match a single expression for readability
   * @type {CompilerExt}
   */
  function compileMatch(mode, _parent) {
    if (!mode.match) return;
    if (mode.begin || mode.end) throw new Error("begin & end are not supported with match");

    mode.begin = mode.match;
    delete mode.match;
  }

  /**
   * provides the default 1 relevance to all modes
   * @type {CompilerExt}
   */
  function compileRelevance(mode, _parent) {
    // eslint-disable-next-line no-undefined
    if (mode.relevance === undefined) mode.relevance = 1;
  }

  // allow beforeMatch to act as a "qualifier" for the match
  // the full match begin must be [beforeMatch][begin]
  const beforeMatchExt = (mode, parent) => {
    if (!mode.beforeMatch) return;
    // starts conflicts with endsParent which we need to make sure the child
    // rule is not matched multiple times
    if (mode.starts) throw new Error("beforeMatch cannot be used with starts");

    const originalMode = Object.assign({}, mode);
    Object.keys(mode).forEach((key) => { delete mode[key]; });

    mode.keywords = originalMode.keywords;
    mode.begin = concat(originalMode.beforeMatch, lookahead(originalMode.begin));
    mode.starts = {
      relevance: 0,
      contains: [
        Object.assign(originalMode, { endsParent: true })
      ]
    };
    mode.relevance = 0;

    delete originalMode.beforeMatch;
  };

  // keywords that should have no default relevance value
  const COMMON_KEYWORDS = [
    'of',
    'and',
    'for',
    'in',
    'not',
    'or',
    'if',
    'then',
    'parent', // common variable name
    'list', // common variable name
    'value' // common variable name
  ];

  const DEFAULT_KEYWORD_SCOPE = "keyword";

  /**
   * Given raw keywords from a language definition, compile them.
   *
   * @param {string | Record<string,string|string[]> | Array<string>} rawKeywords
   * @param {boolean} caseInsensitive
   */
  function compileKeywords(rawKeywords, caseInsensitive, scopeName = DEFAULT_KEYWORD_SCOPE) {
    /** @type {import("highlight.js/private").KeywordDict} */
    const compiledKeywords = Object.create(null);

    // input can be a string of keywords, an array of keywords, or a object with
    // named keys representing scopeName (which can then point to a string or array)
    if (typeof rawKeywords === 'string') {
      compileList(scopeName, rawKeywords.split(" "));
    } else if (Array.isArray(rawKeywords)) {
      compileList(scopeName, rawKeywords);
    } else {
      Object.keys(rawKeywords).forEach(function(scopeName) {
        // collapse all our objects back into the parent object
        Object.assign(
          compiledKeywords,
          compileKeywords(rawKeywords[scopeName], caseInsensitive, scopeName)
        );
      });
    }
    return compiledKeywords;

    // ---

    /**
     * Compiles an individual list of keywords
     *
     * Ex: "for if when while|5"
     *
     * @param {string} scopeName
     * @param {Array<string>} keywordList
     */
    function compileList(scopeName, keywordList) {
      if (caseInsensitive) {
        keywordList = keywordList.map(x => x.toLowerCase());
      }
      keywordList.forEach(function(keyword) {
        const pair = keyword.split('|');
        compiledKeywords[pair[0]] = [scopeName, scoreForKeyword(pair[0], pair[1])];
      });
    }
  }

  /**
   * Returns the proper score for a given keyword
   *
   * Also takes into account comment keywords, which will be scored 0 UNLESS
   * another score has been manually assigned.
   * @param {string} keyword
   * @param {string} [providedScore]
   */
  function scoreForKeyword(keyword, providedScore) {
    // manual scores always win over common keywords
    // so you can force a score of 1 if you really insist
    if (providedScore) {
      return Number(providedScore);
    }

    return commonKeyword(keyword) ? 0 : 1;
  }

  /**
   * Determines if a given keyword is common or not
   *
   * @param {string} keyword */
  function commonKeyword(keyword) {
    return COMMON_KEYWORDS.includes(keyword.toLowerCase());
  }

  /*

  For the reasoning behind this please see:
  https://github.com/highlightjs/highlight.js/issues/2880#issuecomment-747275419

  */

  /**
   * @type {Record<string, boolean>}
   */
  const seenDeprecations = {};

  /**
   * @param {string} message
   */
  const error = (message) => {
    console.error(message);
  };

  /**
   * @param {string} message
   * @param {any} args
   */
  const warn = (message, ...args) => {
    console.log(`WARN: ${message}`, ...args);
  };

  /**
   * @param {string} version
   * @param {string} message
   */
  const deprecated = (version, message) => {
    if (seenDeprecations[`${version}/${message}`]) return;

    console.log(`Deprecated as of ${version}. ${message}`);
    seenDeprecations[`${version}/${message}`] = true;
  };

  /* eslint-disable no-throw-literal */

  /**
  @typedef {import('highlight.js').CompiledMode} CompiledMode
  */

  const MultiClassError = new Error();

  /**
   * Renumbers labeled scope names to account for additional inner match
   * groups that otherwise would break everything.
   *
   * Lets say we 3 match scopes:
   *
   *   { 1 => ..., 2 => ..., 3 => ... }
   *
   * So what we need is a clean match like this:
   *
   *   (a)(b)(c) => [ "a", "b", "c" ]
   *
   * But this falls apart with inner match groups:
   *
   * (a)(((b)))(c) => ["a", "b", "b", "b", "c" ]
   *
   * Our scopes are now "out of alignment" and we're repeating `b` 3 times.
   * What needs to happen is the numbers are remapped:
   *
   *   { 1 => ..., 2 => ..., 5 => ... }
   *
   * We also need to know that the ONLY groups that should be output
   * are 1, 2, and 5.  This function handles this behavior.
   *
   * @param {CompiledMode} mode
   * @param {Array<RegExp | string>} regexes
   * @param {{key: "beginScope"|"endScope"}} opts
   */
  function remapScopeNames(mode, regexes, { key }) {
    let offset = 0;
    const scopeNames = mode[key];
    /** @type Record<number,boolean> */
    const emit = {};
    /** @type Record<number,string> */
    const positions = {};

    for (let i = 1; i <= regexes.length; i++) {
      positions[i + offset] = scopeNames[i];
      emit[i + offset] = true;
      offset += countMatchGroups(regexes[i - 1]);
    }
    // we use _emit to keep track of which match groups are "top-level" to avoid double
    // output from inside match groups
    mode[key] = positions;
    mode[key]._emit = emit;
    mode[key]._multi = true;
  }

  /**
   * @param {CompiledMode} mode
   */
  function beginMultiClass(mode) {
    if (!Array.isArray(mode.begin)) return;

    if (mode.skip || mode.excludeBegin || mode.returnBegin) {
      error("skip, excludeBegin, returnBegin not compatible with beginScope: {}");
      throw MultiClassError;
    }

    if (typeof mode.beginScope !== "object" || mode.beginScope === null) {
      error("beginScope must be object");
      throw MultiClassError;
    }

    remapScopeNames(mode, mode.begin, { key: "beginScope" });
    mode.begin = _rewriteBackreferences(mode.begin, { joinWith: "" });
  }

  /**
   * @param {CompiledMode} mode
   */
  function endMultiClass(mode) {
    if (!Array.isArray(mode.end)) return;

    if (mode.skip || mode.excludeEnd || mode.returnEnd) {
      error("skip, excludeEnd, returnEnd not compatible with endScope: {}");
      throw MultiClassError;
    }

    if (typeof mode.endScope !== "object" || mode.endScope === null) {
      error("endScope must be object");
      throw MultiClassError;
    }

    remapScopeNames(mode, mode.end, { key: "endScope" });
    mode.end = _rewriteBackreferences(mode.end, { joinWith: "" });
  }

  /**
   * this exists only to allow `scope: {}` to be used beside `match:`
   * Otherwise `beginScope` would necessary and that would look weird

    {
      match: [ /def/, /\w+/ ]
      scope: { 1: "keyword" , 2: "title" }
    }

   * @param {CompiledMode} mode
   */
  function scopeSugar(mode) {
    if (mode.scope && typeof mode.scope === "object" && mode.scope !== null) {
      mode.beginScope = mode.scope;
      delete mode.scope;
    }
  }

  /**
   * @param {CompiledMode} mode
   */
  function MultiClass(mode) {
    scopeSugar(mode);

    if (typeof mode.beginScope === "string") {
      mode.beginScope = { _wrap: mode.beginScope };
    }
    if (typeof mode.endScope === "string") {
      mode.endScope = { _wrap: mode.endScope };
    }

    beginMultiClass(mode);
    endMultiClass(mode);
  }

  /**
  @typedef {import('highlight.js').Mode} Mode
  @typedef {import('highlight.js').CompiledMode} CompiledMode
  @typedef {import('highlight.js').Language} Language
  @typedef {import('highlight.js').HLJSPlugin} HLJSPlugin
  @typedef {import('highlight.js').CompiledLanguage} CompiledLanguage
  */

  // compilation

  /**
   * Compiles a language definition result
   *
   * Given the raw result of a language definition (Language), compiles this so
   * that it is ready for highlighting code.
   * @param {Language} language
   * @returns {CompiledLanguage}
   */
  function compileLanguage(language) {
    /**
     * Builds a regex with the case sensitivity of the current language
     *
     * @param {RegExp | string} value
     * @param {boolean} [global]
     */
    function langRe(value, global) {
      return new RegExp(
        source(value),
        'm'
        + (language.case_insensitive ? 'i' : '')
        + (language.unicodeRegex ? 'u' : '')
        + (global ? 'g' : '')
      );
    }

    /**
      Stores multiple regular expressions and allows you to quickly search for
      them all in a string simultaneously - returning the first match.  It does
      this by creating a huge (a|b|c) regex - each individual item wrapped with ()
      and joined by `|` - using match groups to track position.  When a match is
      found checking which position in the array has content allows us to figure
      out which of the original regexes / match groups triggered the match.

      The match object itself (the result of `Regex.exec`) is returned but also
      enhanced by merging in any meta-data that was registered with the regex.
      This is how we keep track of which mode matched, and what type of rule
      (`illegal`, `begin`, end, etc).
    */
    class MultiRegex {
      constructor() {
        this.matchIndexes = {};
        // @ts-ignore
        this.regexes = [];
        this.matchAt = 1;
        this.position = 0;
      }

      // @ts-ignore
      addRule(re, opts) {
        opts.position = this.position++;
        // @ts-ignore
        this.matchIndexes[this.matchAt] = opts;
        this.regexes.push([opts, re]);
        this.matchAt += countMatchGroups(re) + 1;
      }

      compile() {
        if (this.regexes.length === 0) {
          // avoids the need to check length every time exec is called
          // @ts-ignore
          this.exec = () => null;
        }
        const terminators = this.regexes.map(el => el[1]);
        this.matcherRe = langRe(_rewriteBackreferences(terminators, { joinWith: '|' }), true);
        this.lastIndex = 0;
      }

      /** @param {string} s */
      exec(s) {
        this.matcherRe.lastIndex = this.lastIndex;
        const match = this.matcherRe.exec(s);
        if (!match) { return null; }

        // eslint-disable-next-line no-undefined
        const i = match.findIndex((el, i) => i > 0 && el !== undefined);
        // @ts-ignore
        const matchData = this.matchIndexes[i];
        // trim off any earlier non-relevant match groups (ie, the other regex
        // match groups that make up the multi-matcher)
        match.splice(0, i);

        return Object.assign(match, matchData);
      }
    }

    /*
      Created to solve the key deficiently with MultiRegex - there is no way to
      test for multiple matches at a single location.  Why would we need to do
      that?  In the future a more dynamic engine will allow certain matches to be
      ignored.  An example: if we matched say the 3rd regex in a large group but
      decided to ignore it - we'd need to started testing again at the 4th
      regex... but MultiRegex itself gives us no real way to do that.

      So what this class creates MultiRegexs on the fly for whatever search
      position they are needed.

      NOTE: These additional MultiRegex objects are created dynamically.  For most
      grammars most of the time we will never actually need anything more than the
      first MultiRegex - so this shouldn't have too much overhead.

      Say this is our search group, and we match regex3, but wish to ignore it.

        regex1 | regex2 | regex3 | regex4 | regex5    ' ie, startAt = 0

      What we need is a new MultiRegex that only includes the remaining
      possibilities:

        regex4 | regex5                               ' ie, startAt = 3

      This class wraps all that complexity up in a simple API... `startAt` decides
      where in the array of expressions to start doing the matching. It
      auto-increments, so if a match is found at position 2, then startAt will be
      set to 3.  If the end is reached startAt will return to 0.

      MOST of the time the parser will be setting startAt manually to 0.
    */
    class ResumableMultiRegex {
      constructor() {
        // @ts-ignore
        this.rules = [];
        // @ts-ignore
        this.multiRegexes = [];
        this.count = 0;

        this.lastIndex = 0;
        this.regexIndex = 0;
      }

      // @ts-ignore
      getMatcher(index) {
        if (this.multiRegexes[index]) return this.multiRegexes[index];

        const matcher = new MultiRegex();
        this.rules.slice(index).forEach(([re, opts]) => matcher.addRule(re, opts));
        matcher.compile();
        this.multiRegexes[index] = matcher;
        return matcher;
      }

      resumingScanAtSamePosition() {
        return this.regexIndex !== 0;
      }

      considerAll() {
        this.regexIndex = 0;
      }

      // @ts-ignore
      addRule(re, opts) {
        this.rules.push([re, opts]);
        if (opts.type === "begin") this.count++;
      }

      /** @param {string} s */
      exec(s) {
        const m = this.getMatcher(this.regexIndex);
        m.lastIndex = this.lastIndex;
        let result = m.exec(s);

        // The following is because we have no easy way to say "resume scanning at the
        // existing position but also skip the current rule ONLY". What happens is
        // all prior rules are also skipped which can result in matching the wrong
        // thing. Example of matching "booger":

        // our matcher is [string, "booger", number]
        //
        // ....booger....

        // if "booger" is ignored then we'd really need a regex to scan from the
        // SAME position for only: [string, number] but ignoring "booger" (if it
        // was the first match), a simple resume would scan ahead who knows how
        // far looking only for "number", ignoring potential string matches (or
        // future "booger" matches that might be valid.)

        // So what we do: We execute two matchers, one resuming at the same
        // position, but the second full matcher starting at the position after:

        //     /--- resume first regex match here (for [number])
        //     |/---- full match here for [string, "booger", number]
        //     vv
        // ....booger....

        // Which ever results in a match first is then used. So this 3-4 step
        // process essentially allows us to say "match at this position, excluding
        // a prior rule that was ignored".
        //
        // 1. Match "booger" first, ignore. Also proves that [string] does non match.
        // 2. Resume matching for [number]
        // 3. Match at index + 1 for [string, "booger", number]
        // 4. If #2 and #3 result in matches, which came first?
        if (this.resumingScanAtSamePosition()) {
          if (result && result.index === this.lastIndex) ; else { // use the second matcher result
            const m2 = this.getMatcher(0);
            m2.lastIndex = this.lastIndex + 1;
            result = m2.exec(s);
          }
        }

        if (result) {
          this.regexIndex += result.position + 1;
          if (this.regexIndex === this.count) {
            // wrap-around to considering all matches again
            this.considerAll();
          }
        }

        return result;
      }
    }

    /**
     * Given a mode, builds a huge ResumableMultiRegex that can be used to walk
     * the content and find matches.
     *
     * @param {CompiledMode} mode
     * @returns {ResumableMultiRegex}
     */
    function buildModeRegex(mode) {
      const mm = new ResumableMultiRegex();

      mode.contains.forEach(term => mm.addRule(term.begin, { rule: term, type: "begin" }));

      if (mode.terminatorEnd) {
        mm.addRule(mode.terminatorEnd, { type: "end" });
      }
      if (mode.illegal) {
        mm.addRule(mode.illegal, { type: "illegal" });
      }

      return mm;
    }

    /** skip vs abort vs ignore
     *
     * @skip   - The mode is still entered and exited normally (and contains rules apply),
     *           but all content is held and added to the parent buffer rather than being
     *           output when the mode ends.  Mostly used with `sublanguage` to build up
     *           a single large buffer than can be parsed by sublanguage.
     *
     *             - The mode begin ands ends normally.
     *             - Content matched is added to the parent mode buffer.
     *             - The parser cursor is moved forward normally.
     *
     * @abort  - A hack placeholder until we have ignore.  Aborts the mode (as if it
     *           never matched) but DOES NOT continue to match subsequent `contains`
     *           modes.  Abort is bad/suboptimal because it can result in modes
     *           farther down not getting applied because an earlier rule eats the
     *           content but then aborts.
     *
     *             - The mode does not begin.
     *             - Content matched by `begin` is added to the mode buffer.
     *             - The parser cursor is moved forward accordingly.
     *
     * @ignore - Ignores the mode (as if it never matched) and continues to match any
     *           subsequent `contains` modes.  Ignore isn't technically possible with
     *           the current parser implementation.
     *
     *             - The mode does not begin.
     *             - Content matched by `begin` is ignored.
     *             - The parser cursor is not moved forward.
     */

    /**
     * Compiles an individual mode
     *
     * This can raise an error if the mode contains certain detectable known logic
     * issues.
     * @param {Mode} mode
     * @param {CompiledMode | null} [parent]
     * @returns {CompiledMode | never}
     */
    function compileMode(mode, parent) {
      const cmode = /** @type CompiledMode */ (mode);
      if (mode.isCompiled) return cmode;

      [
        scopeClassName,
        // do this early so compiler extensions generally don't have to worry about
        // the distinction between match/begin
        compileMatch,
        MultiClass,
        beforeMatchExt
      ].forEach(ext => ext(mode, parent));

      language.compilerExtensions.forEach(ext => ext(mode, parent));

      // __beforeBegin is considered private API, internal use only
      mode.__beforeBegin = null;

      [
        beginKeywords,
        // do this later so compiler extensions that come earlier have access to the
        // raw array if they wanted to perhaps manipulate it, etc.
        compileIllegal,
        // default to 1 relevance if not specified
        compileRelevance
      ].forEach(ext => ext(mode, parent));

      mode.isCompiled = true;

      let keywordPattern = null;
      if (typeof mode.keywords === "object" && mode.keywords.$pattern) {
        // we need a copy because keywords might be compiled multiple times
        // so we can't go deleting $pattern from the original on the first
        // pass
        mode.keywords = Object.assign({}, mode.keywords);
        keywordPattern = mode.keywords.$pattern;
        delete mode.keywords.$pattern;
      }
      keywordPattern = keywordPattern || /\w+/;

      if (mode.keywords) {
        mode.keywords = compileKeywords(mode.keywords, language.case_insensitive);
      }

      cmode.keywordPatternRe = langRe(keywordPattern, true);

      if (parent) {
        if (!mode.begin) mode.begin = /\B|\b/;
        cmode.beginRe = langRe(cmode.begin);
        if (!mode.end && !mode.endsWithParent) mode.end = /\B|\b/;
        if (mode.end) cmode.endRe = langRe(cmode.end);
        cmode.terminatorEnd = source(cmode.end) || '';
        if (mode.endsWithParent && parent.terminatorEnd) {
          cmode.terminatorEnd += (mode.end ? '|' : '') + parent.terminatorEnd;
        }
      }
      if (mode.illegal) cmode.illegalRe = langRe(/** @type {RegExp | string} */ (mode.illegal));
      if (!mode.contains) mode.contains = [];

      mode.contains = [].concat(...mode.contains.map(function(c) {
        return expandOrCloneMode(c === 'self' ? mode : c);
      }));
      mode.contains.forEach(function(c) { compileMode(/** @type Mode */ (c), cmode); });

      if (mode.starts) {
        compileMode(mode.starts, parent);
      }

      cmode.matcher = buildModeRegex(cmode);
      return cmode;
    }

    if (!language.compilerExtensions) language.compilerExtensions = [];

    // self is not valid at the top-level
    if (language.contains && language.contains.includes('self')) {
      throw new Error("ERR: contains `self` is not supported at the top-level of a language.  See documentation.");
    }

    // we need a null object, which inherit will guarantee
    language.classNameAliases = inherit$1(language.classNameAliases || {});

    return compileMode(/** @type Mode */ (language));
  }

  /**
   * Determines if a mode has a dependency on it's parent or not
   *
   * If a mode does have a parent dependency then often we need to clone it if
   * it's used in multiple places so that each copy points to the correct parent,
   * where-as modes without a parent can often safely be re-used at the bottom of
   * a mode chain.
   *
   * @param {Mode | null} mode
   * @returns {boolean} - is there a dependency on the parent?
   * */
  function dependencyOnParent(mode) {
    if (!mode) return false;

    return mode.endsWithParent || dependencyOnParent(mode.starts);
  }

  /**
   * Expands a mode or clones it if necessary
   *
   * This is necessary for modes with parental dependenceis (see notes on
   * `dependencyOnParent`) and for nodes that have `variants` - which must then be
   * exploded into their own individual modes at compile time.
   *
   * @param {Mode} mode
   * @returns {Mode | Mode[]}
   * */
  function expandOrCloneMode(mode) {
    if (mode.variants && !mode.cachedVariants) {
      mode.cachedVariants = mode.variants.map(function(variant) {
        return inherit$1(mode, { variants: null }, variant);
      });
    }

    // EXPAND
    // if we have variants then essentially "replace" the mode with the variants
    // this happens in compileMode, where this function is called from
    if (mode.cachedVariants) {
      return mode.cachedVariants;
    }

    // CLONE
    // if we have dependencies on parents then we need a unique
    // instance of ourselves, so we can be reused with many
    // different parents without issue
    if (dependencyOnParent(mode)) {
      return inherit$1(mode, { starts: mode.starts ? inherit$1(mode.starts) : null });
    }

    if (Object.isFrozen(mode)) {
      return inherit$1(mode);
    }

    // no special dependency issues, just return ourselves
    return mode;
  }

  var version = "11.10.0";

  class HTMLInjectionError extends Error {
    constructor(reason, html) {
      super(reason);
      this.name = "HTMLInjectionError";
      this.html = html;
    }
  }

  /*
  Syntax highlighting with language autodetection.
  https://highlightjs.org/
  */



  /**
  @typedef {import('highlight.js').Mode} Mode
  @typedef {import('highlight.js').CompiledMode} CompiledMode
  @typedef {import('highlight.js').CompiledScope} CompiledScope
  @typedef {import('highlight.js').Language} Language
  @typedef {import('highlight.js').HLJSApi} HLJSApi
  @typedef {import('highlight.js').HLJSPlugin} HLJSPlugin
  @typedef {import('highlight.js').PluginEvent} PluginEvent
  @typedef {import('highlight.js').HLJSOptions} HLJSOptions
  @typedef {import('highlight.js').LanguageFn} LanguageFn
  @typedef {import('highlight.js').HighlightedHTMLElement} HighlightedHTMLElement
  @typedef {import('highlight.js').BeforeHighlightContext} BeforeHighlightContext
  @typedef {import('highlight.js/private').MatchType} MatchType
  @typedef {import('highlight.js/private').KeywordData} KeywordData
  @typedef {import('highlight.js/private').EnhancedMatch} EnhancedMatch
  @typedef {import('highlight.js/private').AnnotatedError} AnnotatedError
  @typedef {import('highlight.js').AutoHighlightResult} AutoHighlightResult
  @typedef {import('highlight.js').HighlightOptions} HighlightOptions
  @typedef {import('highlight.js').HighlightResult} HighlightResult
  */


  const escape = escapeHTML;
  const inherit = inherit$1;
  const NO_MATCH = Symbol("nomatch");
  const MAX_KEYWORD_HITS = 7;

  /**
   * @param {any} hljs - object that is extended (legacy)
   * @returns {HLJSApi}
   */
  const HLJS = function(hljs) {
    // Global internal variables used within the highlight.js library.
    /** @type {Record<string, Language>} */
    const languages = Object.create(null);
    /** @type {Record<string, string>} */
    const aliases = Object.create(null);
    /** @type {HLJSPlugin[]} */
    const plugins = [];

    // safe/production mode - swallows more errors, tries to keep running
    // even if a single syntax or parse hits a fatal error
    let SAFE_MODE = true;
    const LANGUAGE_NOT_FOUND = "Could not find the language '{}', did you forget to load/include a language module?";
    /** @type {Language} */
    const PLAINTEXT_LANGUAGE = { disableAutodetect: true, name: 'Plain text', contains: [] };

    // Global options used when within external APIs. This is modified when
    // calling the `hljs.configure` function.
    /** @type HLJSOptions */
    let options = {
      ignoreUnescapedHTML: false,
      throwUnescapedHTML: false,
      noHighlightRe: /^(no-?highlight)$/i,
      languageDetectRe: /\blang(?:uage)?-([\w-]+)\b/i,
      classPrefix: 'hljs-',
      cssSelector: 'pre code',
      languages: null,
      // beta configuration options, subject to change, welcome to discuss
      // https://github.com/highlightjs/highlight.js/issues/1086
      __emitter: TokenTreeEmitter
    };

    /* Utility functions */

    /**
     * Tests a language name to see if highlighting should be skipped
     * @param {string} languageName
     */
    function shouldNotHighlight(languageName) {
      return options.noHighlightRe.test(languageName);
    }

    /**
     * @param {HighlightedHTMLElement} block - the HTML element to determine language for
     */
    function blockLanguage(block) {
      let classes = block.className + ' ';

      classes += block.parentNode ? block.parentNode.className : '';

      // language-* takes precedence over non-prefixed class names.
      const match = options.languageDetectRe.exec(classes);
      if (match) {
        const language = getLanguage(match[1]);
        if (!language) {
          warn(LANGUAGE_NOT_FOUND.replace("{}", match[1]));
          warn("Falling back to no-highlight mode for this block.", block);
        }
        return language ? match[1] : 'no-highlight';
      }

      return classes
        .split(/\s+/)
        .find((_class) => shouldNotHighlight(_class) || getLanguage(_class));
    }

    /**
     * Core highlighting function.
     *
     * OLD API
     * highlight(lang, code, ignoreIllegals, continuation)
     *
     * NEW API
     * highlight(code, {lang, ignoreIllegals})
     *
     * @param {string} codeOrLanguageName - the language to use for highlighting
     * @param {string | HighlightOptions} optionsOrCode - the code to highlight
     * @param {boolean} [ignoreIllegals] - whether to ignore illegal matches, default is to bail
     *
     * @returns {HighlightResult} Result - an object that represents the result
     * @property {string} language - the language name
     * @property {number} relevance - the relevance score
     * @property {string} value - the highlighted HTML code
     * @property {string} code - the original raw code
     * @property {CompiledMode} top - top of the current mode stack
     * @property {boolean} illegal - indicates whether any illegal matches were found
    */
    function highlight(codeOrLanguageName, optionsOrCode, ignoreIllegals) {
      let code = "";
      let languageName = "";
      if (typeof optionsOrCode === "object") {
        code = codeOrLanguageName;
        ignoreIllegals = optionsOrCode.ignoreIllegals;
        languageName = optionsOrCode.language;
      } else {
        // old API
        deprecated("10.7.0", "highlight(lang, code, ...args) has been deprecated.");
        deprecated("10.7.0", "Please use highlight(code, options) instead.\nhttps://github.com/highlightjs/highlight.js/issues/2277");
        languageName = codeOrLanguageName;
        code = optionsOrCode;
      }

      // https://github.com/highlightjs/highlight.js/issues/3149
      // eslint-disable-next-line no-undefined
      if (ignoreIllegals === undefined) { ignoreIllegals = true; }

      /** @type {BeforeHighlightContext} */
      const context = {
        code,
        language: languageName
      };
      // the plugin can change the desired language or the code to be highlighted
      // just be changing the object it was passed
      fire("before:highlight", context);

      // a before plugin can usurp the result completely by providing it's own
      // in which case we don't even need to call highlight
      const result = context.result
        ? context.result
        : _highlight(context.language, context.code, ignoreIllegals);

      result.code = context.code;
      // the plugin can change anything in result to suite it
      fire("after:highlight", result);

      return result;
    }

    /**
     * private highlight that's used internally and does not fire callbacks
     *
     * @param {string} languageName - the language to use for highlighting
     * @param {string} codeToHighlight - the code to highlight
     * @param {boolean?} [ignoreIllegals] - whether to ignore illegal matches, default is to bail
     * @param {CompiledMode?} [continuation] - current continuation mode, if any
     * @returns {HighlightResult} - result of the highlight operation
    */
    function _highlight(languageName, codeToHighlight, ignoreIllegals, continuation) {
      const keywordHits = Object.create(null);

      /**
       * Return keyword data if a match is a keyword
       * @param {CompiledMode} mode - current mode
       * @param {string} matchText - the textual match
       * @returns {KeywordData | false}
       */
      function keywordData(mode, matchText) {
        return mode.keywords[matchText];
      }

      function processKeywords() {
        if (!top.keywords) {
          emitter.addText(modeBuffer);
          return;
        }

        let lastIndex = 0;
        top.keywordPatternRe.lastIndex = 0;
        let match = top.keywordPatternRe.exec(modeBuffer);
        let buf = "";

        while (match) {
          buf += modeBuffer.substring(lastIndex, match.index);
          const word = language.case_insensitive ? match[0].toLowerCase() : match[0];
          const data = keywordData(top, word);
          if (data) {
            const [kind, keywordRelevance] = data;
            emitter.addText(buf);
            buf = "";

            keywordHits[word] = (keywordHits[word] || 0) + 1;
            if (keywordHits[word] <= MAX_KEYWORD_HITS) relevance += keywordRelevance;
            if (kind.startsWith("_")) {
              // _ implied for relevance only, do not highlight
              // by applying a class name
              buf += match[0];
            } else {
              const cssClass = language.classNameAliases[kind] || kind;
              emitKeyword(match[0], cssClass);
            }
          } else {
            buf += match[0];
          }
          lastIndex = top.keywordPatternRe.lastIndex;
          match = top.keywordPatternRe.exec(modeBuffer);
        }
        buf += modeBuffer.substring(lastIndex);
        emitter.addText(buf);
      }

      function processSubLanguage() {
        if (modeBuffer === "") return;
        /** @type HighlightResult */
        let result = null;

        if (typeof top.subLanguage === 'string') {
          if (!languages[top.subLanguage]) {
            emitter.addText(modeBuffer);
            return;
          }
          result = _highlight(top.subLanguage, modeBuffer, true, continuations[top.subLanguage]);
          continuations[top.subLanguage] = /** @type {CompiledMode} */ (result._top);
        } else {
          result = highlightAuto(modeBuffer, top.subLanguage.length ? top.subLanguage : null);
        }

        // Counting embedded language score towards the host language may be disabled
        // with zeroing the containing mode relevance. Use case in point is Markdown that
        // allows XML everywhere and makes every XML snippet to have a much larger Markdown
        // score.
        if (top.relevance > 0) {
          relevance += result.relevance;
        }
        emitter.__addSublanguage(result._emitter, result.language);
      }

      function processBuffer() {
        if (top.subLanguage != null) {
          processSubLanguage();
        } else {
          processKeywords();
        }
        modeBuffer = '';
      }

      /**
       * @param {string} text
       * @param {string} scope
       */
      function emitKeyword(keyword, scope) {
        if (keyword === "") return;

        emitter.startScope(scope);
        emitter.addText(keyword);
        emitter.endScope();
      }

      /**
       * @param {CompiledScope} scope
       * @param {RegExpMatchArray} match
       */
      function emitMultiClass(scope, match) {
        let i = 1;
        const max = match.length - 1;
        while (i <= max) {
          if (!scope._emit[i]) { i++; continue; }
          const klass = language.classNameAliases[scope[i]] || scope[i];
          const text = match[i];
          if (klass) {
            emitKeyword(text, klass);
          } else {
            modeBuffer = text;
            processKeywords();
            modeBuffer = "";
          }
          i++;
        }
      }

      /**
       * @param {CompiledMode} mode - new mode to start
       * @param {RegExpMatchArray} match
       */
      function startNewMode(mode, match) {
        if (mode.scope && typeof mode.scope === "string") {
          emitter.openNode(language.classNameAliases[mode.scope] || mode.scope);
        }
        if (mode.beginScope) {
          // beginScope just wraps the begin match itself in a scope
          if (mode.beginScope._wrap) {
            emitKeyword(modeBuffer, language.classNameAliases[mode.beginScope._wrap] || mode.beginScope._wrap);
            modeBuffer = "";
          } else if (mode.beginScope._multi) {
            // at this point modeBuffer should just be the match
            emitMultiClass(mode.beginScope, match);
            modeBuffer = "";
          }
        }

        top = Object.create(mode, { parent: { value: top } });
        return top;
      }

      /**
       * @param {CompiledMode } mode - the mode to potentially end
       * @param {RegExpMatchArray} match - the latest match
       * @param {string} matchPlusRemainder - match plus remainder of content
       * @returns {CompiledMode | void} - the next mode, or if void continue on in current mode
       */
      function endOfMode(mode, match, matchPlusRemainder) {
        let matched = startsWith(mode.endRe, matchPlusRemainder);

        if (matched) {
          if (mode["on:end"]) {
            const resp = new Response(mode);
            mode["on:end"](match, resp);
            if (resp.isMatchIgnored) matched = false;
          }

          if (matched) {
            while (mode.endsParent && mode.parent) {
              mode = mode.parent;
            }
            return mode;
          }
        }
        // even if on:end fires an `ignore` it's still possible
        // that we might trigger the end node because of a parent mode
        if (mode.endsWithParent) {
          return endOfMode(mode.parent, match, matchPlusRemainder);
        }
      }

      /**
       * Handle matching but then ignoring a sequence of text
       *
       * @param {string} lexeme - string containing full match text
       */
      function doIgnore(lexeme) {
        if (top.matcher.regexIndex === 0) {
          // no more regexes to potentially match here, so we move the cursor forward one
          // space
          modeBuffer += lexeme[0];
          return 1;
        } else {
          // no need to move the cursor, we still have additional regexes to try and
          // match at this very spot
          resumeScanAtSamePosition = true;
          return 0;
        }
      }

      /**
       * Handle the start of a new potential mode match
       *
       * @param {EnhancedMatch} match - the current match
       * @returns {number} how far to advance the parse cursor
       */
      function doBeginMatch(match) {
        const lexeme = match[0];
        const newMode = match.rule;

        const resp = new Response(newMode);
        // first internal before callbacks, then the public ones
        const beforeCallbacks = [newMode.__beforeBegin, newMode["on:begin"]];
        for (const cb of beforeCallbacks) {
          if (!cb) continue;
          cb(match, resp);
          if (resp.isMatchIgnored) return doIgnore(lexeme);
        }

        if (newMode.skip) {
          modeBuffer += lexeme;
        } else {
          if (newMode.excludeBegin) {
            modeBuffer += lexeme;
          }
          processBuffer();
          if (!newMode.returnBegin && !newMode.excludeBegin) {
            modeBuffer = lexeme;
          }
        }
        startNewMode(newMode, match);
        return newMode.returnBegin ? 0 : lexeme.length;
      }

      /**
       * Handle the potential end of mode
       *
       * @param {RegExpMatchArray} match - the current match
       */
      function doEndMatch(match) {
        const lexeme = match[0];
        const matchPlusRemainder = codeToHighlight.substring(match.index);

        const endMode = endOfMode(top, match, matchPlusRemainder);
        if (!endMode) { return NO_MATCH; }

        const origin = top;
        if (top.endScope && top.endScope._wrap) {
          processBuffer();
          emitKeyword(lexeme, top.endScope._wrap);
        } else if (top.endScope && top.endScope._multi) {
          processBuffer();
          emitMultiClass(top.endScope, match);
        } else if (origin.skip) {
          modeBuffer += lexeme;
        } else {
          if (!(origin.returnEnd || origin.excludeEnd)) {
            modeBuffer += lexeme;
          }
          processBuffer();
          if (origin.excludeEnd) {
            modeBuffer = lexeme;
          }
        }
        do {
          if (top.scope) {
            emitter.closeNode();
          }
          if (!top.skip && !top.subLanguage) {
            relevance += top.relevance;
          }
          top = top.parent;
        } while (top !== endMode.parent);
        if (endMode.starts) {
          startNewMode(endMode.starts, match);
        }
        return origin.returnEnd ? 0 : lexeme.length;
      }

      function processContinuations() {
        const list = [];
        for (let current = top; current !== language; current = current.parent) {
          if (current.scope) {
            list.unshift(current.scope);
          }
        }
        list.forEach(item => emitter.openNode(item));
      }

      /** @type {{type?: MatchType, index?: number, rule?: Mode}}} */
      let lastMatch = {};

      /**
       *  Process an individual match
       *
       * @param {string} textBeforeMatch - text preceding the match (since the last match)
       * @param {EnhancedMatch} [match] - the match itself
       */
      function processLexeme(textBeforeMatch, match) {
        const lexeme = match && match[0];

        // add non-matched text to the current mode buffer
        modeBuffer += textBeforeMatch;

        if (lexeme == null) {
          processBuffer();
          return 0;
        }

        // we've found a 0 width match and we're stuck, so we need to advance
        // this happens when we have badly behaved rules that have optional matchers to the degree that
        // sometimes they can end up matching nothing at all
        // Ref: https://github.com/highlightjs/highlight.js/issues/2140
        if (lastMatch.type === "begin" && match.type === "end" && lastMatch.index === match.index && lexeme === "") {
          // spit the "skipped" character that our regex choked on back into the output sequence
          modeBuffer += codeToHighlight.slice(match.index, match.index + 1);
          if (!SAFE_MODE) {
            /** @type {AnnotatedError} */
            const err = new Error(`0 width match regex (${languageName})`);
            err.languageName = languageName;
            err.badRule = lastMatch.rule;
            throw err;
          }
          return 1;
        }
        lastMatch = match;

        if (match.type === "begin") {
          return doBeginMatch(match);
        } else if (match.type === "illegal" && !ignoreIllegals) {
          // illegal match, we do not continue processing
          /** @type {AnnotatedError} */
          const err = new Error('Illegal lexeme "' + lexeme + '" for mode "' + (top.scope || '<unnamed>') + '"');
          err.mode = top;
          throw err;
        } else if (match.type === "end") {
          const processed = doEndMatch(match);
          if (processed !== NO_MATCH) {
            return processed;
          }
        }

        // edge case for when illegal matches $ (end of line) which is technically
        // a 0 width match but not a begin/end match so it's not caught by the
        // first handler (when ignoreIllegals is true)
        if (match.type === "illegal" && lexeme === "") {
          // advance so we aren't stuck in an infinite loop
          return 1;
        }

        // infinite loops are BAD, this is a last ditch catch all. if we have a
        // decent number of iterations yet our index (cursor position in our
        // parsing) still 3x behind our index then something is very wrong
        // so we bail
        if (iterations > 100000 && iterations > match.index * 3) {
          const err = new Error('potential infinite loop, way more iterations than matches');
          throw err;
        }

        /*
        Why might be find ourselves here?  An potential end match that was
        triggered but could not be completed.  IE, `doEndMatch` returned NO_MATCH.
        (this could be because a callback requests the match be ignored, etc)

        This causes no real harm other than stopping a few times too many.
        */

        modeBuffer += lexeme;
        return lexeme.length;
      }

      const language = getLanguage(languageName);
      if (!language) {
        error(LANGUAGE_NOT_FOUND.replace("{}", languageName));
        throw new Error('Unknown language: "' + languageName + '"');
      }

      const md = compileLanguage(language);
      let result = '';
      /** @type {CompiledMode} */
      let top = continuation || md;
      /** @type Record<string,CompiledMode> */
      const continuations = {}; // keep continuations for sub-languages
      const emitter = new options.__emitter(options);
      processContinuations();
      let modeBuffer = '';
      let relevance = 0;
      let index = 0;
      let iterations = 0;
      let resumeScanAtSamePosition = false;

      try {
        if (!language.__emitTokens) {
          top.matcher.considerAll();

          for (;;) {
            iterations++;
            if (resumeScanAtSamePosition) {
              // only regexes not matched previously will now be
              // considered for a potential match
              resumeScanAtSamePosition = false;
            } else {
              top.matcher.considerAll();
            }
            top.matcher.lastIndex = index;

            const match = top.matcher.exec(codeToHighlight);
            // console.log("match", match[0], match.rule && match.rule.begin)

            if (!match) break;

            const beforeMatch = codeToHighlight.substring(index, match.index);
            const processedCount = processLexeme(beforeMatch, match);
            index = match.index + processedCount;
          }
          processLexeme(codeToHighlight.substring(index));
        } else {
          language.__emitTokens(codeToHighlight, emitter);
        }

        emitter.finalize();
        result = emitter.toHTML();

        return {
          language: languageName,
          value: result,
          relevance,
          illegal: false,
          _emitter: emitter,
          _top: top
        };
      } catch (err) {
        if (err.message && err.message.includes('Illegal')) {
          return {
            language: languageName,
            value: escape(codeToHighlight),
            illegal: true,
            relevance: 0,
            _illegalBy: {
              message: err.message,
              index,
              context: codeToHighlight.slice(index - 100, index + 100),
              mode: err.mode,
              resultSoFar: result
            },
            _emitter: emitter
          };
        } else if (SAFE_MODE) {
          return {
            language: languageName,
            value: escape(codeToHighlight),
            illegal: false,
            relevance: 0,
            errorRaised: err,
            _emitter: emitter,
            _top: top
          };
        } else {
          throw err;
        }
      }
    }

    /**
     * returns a valid highlight result, without actually doing any actual work,
     * auto highlight starts with this and it's possible for small snippets that
     * auto-detection may not find a better match
     * @param {string} code
     * @returns {HighlightResult}
     */
    function justTextHighlightResult(code) {
      const result = {
        value: escape(code),
        illegal: false,
        relevance: 0,
        _top: PLAINTEXT_LANGUAGE,
        _emitter: new options.__emitter(options)
      };
      result._emitter.addText(code);
      return result;
    }

    /**
    Highlighting with language detection. Accepts a string with the code to
    highlight. Returns an object with the following properties:

    - language (detected language)
    - relevance (int)
    - value (an HTML string with highlighting markup)
    - secondBest (object with the same structure for second-best heuristically
      detected language, may be absent)

      @param {string} code
      @param {Array<string>} [languageSubset]
      @returns {AutoHighlightResult}
    */
    function highlightAuto(code, languageSubset) {
      languageSubset = languageSubset || options.languages || Object.keys(languages);
      const plaintext = justTextHighlightResult(code);

      const results = languageSubset.filter(getLanguage).filter(autoDetection).map(name =>
        _highlight(name, code, false)
      );
      results.unshift(plaintext); // plaintext is always an option

      const sorted = results.sort((a, b) => {
        // sort base on relevance
        if (a.relevance !== b.relevance) return b.relevance - a.relevance;

        // always award the tie to the base language
        // ie if C++ and Arduino are tied, it's more likely to be C++
        if (a.language && b.language) {
          if (getLanguage(a.language).supersetOf === b.language) {
            return 1;
          } else if (getLanguage(b.language).supersetOf === a.language) {
            return -1;
          }
        }

        // otherwise say they are equal, which has the effect of sorting on
        // relevance while preserving the original ordering - which is how ties
        // have historically been settled, ie the language that comes first always
        // wins in the case of a tie
        return 0;
      });

      const [best, secondBest] = sorted;

      /** @type {AutoHighlightResult} */
      const result = best;
      result.secondBest = secondBest;

      return result;
    }

    /**
     * Builds new class name for block given the language name
     *
     * @param {HTMLElement} element
     * @param {string} [currentLang]
     * @param {string} [resultLang]
     */
    function updateClassName(element, currentLang, resultLang) {
      const language = (currentLang && aliases[currentLang]) || resultLang;

      element.classList.add("hljs");
      element.classList.add(`language-${language}`);
    }

    /**
     * Applies highlighting to a DOM node containing code.
     *
     * @param {HighlightedHTMLElement} element - the HTML element to highlight
    */
    function highlightElement(element) {
      /** @type HTMLElement */
      let node = null;
      const language = blockLanguage(element);

      if (shouldNotHighlight(language)) return;

      fire("before:highlightElement",
        { el: element, language });

      if (element.dataset.highlighted) {
        console.log("Element previously highlighted. To highlight again, first unset `dataset.highlighted`.", element);
        return;
      }

      // we should be all text, no child nodes (unescaped HTML) - this is possibly
      // an HTML injection attack - it's likely too late if this is already in
      // production (the code has likely already done its damage by the time
      // we're seeing it)... but we yell loudly about this so that hopefully it's
      // more likely to be caught in development before making it to production
      if (element.children.length > 0) {
        if (!options.ignoreUnescapedHTML) {
          console.warn("One of your code blocks includes unescaped HTML. This is a potentially serious security risk.");
          console.warn("https://github.com/highlightjs/highlight.js/wiki/security");
          console.warn("The element with unescaped HTML:");
          console.warn(element);
        }
        if (options.throwUnescapedHTML) {
          const err = new HTMLInjectionError(
            "One of your code blocks includes unescaped HTML.",
            element.innerHTML
          );
          throw err;
        }
      }

      node = element;
      const text = node.textContent;
      const result = language ? highlight(text, { language, ignoreIllegals: true }) : highlightAuto(text);

      element.innerHTML = result.value;
      element.dataset.highlighted = "yes";
      updateClassName(element, language, result.language);
      element.result = {
        language: result.language,
        // TODO: remove with version 11.0
        re: result.relevance,
        relevance: result.relevance
      };
      if (result.secondBest) {
        element.secondBest = {
          language: result.secondBest.language,
          relevance: result.secondBest.relevance
        };
      }

      fire("after:highlightElement", { el: element, result, text });
    }

    /**
     * Updates highlight.js global options with the passed options
     *
     * @param {Partial<HLJSOptions>} userOptions
     */
    function configure(userOptions) {
      options = inherit(options, userOptions);
    }

    // TODO: remove v12, deprecated
    const initHighlighting = () => {
      highlightAll();
      deprecated("10.6.0", "initHighlighting() deprecated.  Use highlightAll() now.");
    };

    // TODO: remove v12, deprecated
    function initHighlightingOnLoad() {
      highlightAll();
      deprecated("10.6.0", "initHighlightingOnLoad() deprecated.  Use highlightAll() now.");
    }

    let wantsHighlight = false;

    /**
     * auto-highlights all pre>code elements on the page
     */
    function highlightAll() {
      // if we are called too early in the loading process
      if (document.readyState === "loading") {
        wantsHighlight = true;
        return;
      }

      const blocks = document.querySelectorAll(options.cssSelector);
      blocks.forEach(highlightElement);
    }

    function boot() {
      // if a highlight was requested before DOM was loaded, do now
      if (wantsHighlight) highlightAll();
    }

    // make sure we are in the browser environment
    if (typeof window !== 'undefined' && window.addEventListener) {
      window.addEventListener('DOMContentLoaded', boot, false);
    }

    /**
     * Register a language grammar module
     *
     * @param {string} languageName
     * @param {LanguageFn} languageDefinition
     */
    function registerLanguage(languageName, languageDefinition) {
      let lang = null;
      try {
        lang = languageDefinition(hljs);
      } catch (error$1) {
        error("Language definition for '{}' could not be registered.".replace("{}", languageName));
        // hard or soft error
        if (!SAFE_MODE) { throw error$1; } else { error(error$1); }
        // languages that have serious errors are replaced with essentially a
        // "plaintext" stand-in so that the code blocks will still get normal
        // css classes applied to them - and one bad language won't break the
        // entire highlighter
        lang = PLAINTEXT_LANGUAGE;
      }
      // give it a temporary name if it doesn't have one in the meta-data
      if (!lang.name) lang.name = languageName;
      languages[languageName] = lang;
      lang.rawDefinition = languageDefinition.bind(null, hljs);

      if (lang.aliases) {
        registerAliases(lang.aliases, { languageName });
      }
    }

    /**
     * Remove a language grammar module
     *
     * @param {string} languageName
     */
    function unregisterLanguage(languageName) {
      delete languages[languageName];
      for (const alias of Object.keys(aliases)) {
        if (aliases[alias] === languageName) {
          delete aliases[alias];
        }
      }
    }

    /**
     * @returns {string[]} List of language internal names
     */
    function listLanguages() {
      return Object.keys(languages);
    }

    /**
     * @param {string} name - name of the language to retrieve
     * @returns {Language | undefined}
     */
    function getLanguage(name) {
      name = (name || '').toLowerCase();
      return languages[name] || languages[aliases[name]];
    }

    /**
     *
     * @param {string|string[]} aliasList - single alias or list of aliases
     * @param {{languageName: string}} opts
     */
    function registerAliases(aliasList, { languageName }) {
      if (typeof aliasList === 'string') {
        aliasList = [aliasList];
      }
      aliasList.forEach(alias => { aliases[alias.toLowerCase()] = languageName; });
    }

    /**
     * Determines if a given language has auto-detection enabled
     * @param {string} name - name of the language
     */
    function autoDetection(name) {
      const lang = getLanguage(name);
      return lang && !lang.disableAutodetect;
    }

    /**
     * Upgrades the old highlightBlock plugins to the new
     * highlightElement API
     * @param {HLJSPlugin} plugin
     */
    function upgradePluginAPI(plugin) {
      // TODO: remove with v12
      if (plugin["before:highlightBlock"] && !plugin["before:highlightElement"]) {
        plugin["before:highlightElement"] = (data) => {
          plugin["before:highlightBlock"](
            Object.assign({ block: data.el }, data)
          );
        };
      }
      if (plugin["after:highlightBlock"] && !plugin["after:highlightElement"]) {
        plugin["after:highlightElement"] = (data) => {
          plugin["after:highlightBlock"](
            Object.assign({ block: data.el }, data)
          );
        };
      }
    }

    /**
     * @param {HLJSPlugin} plugin
     */
    function addPlugin(plugin) {
      upgradePluginAPI(plugin);
      plugins.push(plugin);
    }

    /**
     * @param {HLJSPlugin} plugin
     */
    function removePlugin(plugin) {
      const index = plugins.indexOf(plugin);
      if (index !== -1) {
        plugins.splice(index, 1);
      }
    }

    /**
     *
     * @param {PluginEvent} event
     * @param {any} args
     */
    function fire(event, args) {
      const cb = event;
      plugins.forEach(function(plugin) {
        if (plugin[cb]) {
          plugin[cb](args);
        }
      });
    }

    /**
     * DEPRECATED
     * @param {HighlightedHTMLElement} el
     */
    function deprecateHighlightBlock(el) {
      deprecated("10.7.0", "highlightBlock will be removed entirely in v12.0");
      deprecated("10.7.0", "Please use highlightElement now.");

      return highlightElement(el);
    }

    /* Interface definition */
    Object.assign(hljs, {
      highlight,
      highlightAuto,
      highlightAll,
      highlightElement,
      // TODO: Remove with v12 API
      highlightBlock: deprecateHighlightBlock,
      configure,
      initHighlighting,
      initHighlightingOnLoad,
      registerLanguage,
      unregisterLanguage,
      listLanguages,
      getLanguage,
      registerAliases,
      autoDetection,
      inherit,
      addPlugin,
      removePlugin
    });

    hljs.debugMode = function() { SAFE_MODE = false; };
    hljs.safeMode = function() { SAFE_MODE = true; };
    hljs.versionString = version;

    hljs.regex = {
      concat: concat,
      lookahead: lookahead,
      either: either,
      optional: optional,
      anyNumberOfTimes: anyNumberOfTimes
    };

    for (const key in MODES) {
      // @ts-ignore
      if (typeof MODES[key] === "object") {
        // @ts-ignore
        deepFreeze(MODES[key]);
      }
    }

    // merge all the modes/regexes into our main object
    Object.assign(hljs, MODES);

    return hljs;
  };

  // Other names for the variable may break build script
  const highlight = HLJS({});

  // returns a new instance of the highlighter to be used for extensions
  // check https://github.com/wooorm/lowlight/issues/47
  highlight.newInstance = () => HLJS({});

  return highlight;

})();
if (typeof exports === 'object' && typeof module !== 'undefined') { module.exports = hljs; }
/*! `bash` grammar compiled for Highlight.js 11.10.0 */
  (function(){
    var hljsGrammar = (function () {
  'use strict';

  /*
  Language: Bash
  Author: vah <vahtenberg@gmail.com>
  Contributrors: Benjamin Pannell <contact@sierrasoftworks.com>
  Website: https://www.gnu.org/software/bash/
  Category: common, scripting
  */

  /** @type LanguageFn */
  function bash(hljs) {
    const regex = hljs.regex;
    const VAR = {};
    const BRACED_VAR = {
      begin: /\$\{/,
      end: /\}/,
      contains: [
        "self",
        {
          begin: /:-/,
          contains: [ VAR ]
        } // default values
      ]
    };
    Object.assign(VAR, {
      className: 'variable',
      variants: [
        { begin: regex.concat(/\$[\w\d#@][\w\d_]*/,
          // negative look-ahead tries to avoid matching patterns that are not
          // Perl at all like $ident$, @ident@, etc.
          `(?![\\w\\d])(?![$])`) },
        BRACED_VAR
      ]
    });

    const SUBST = {
      className: 'subst',
      begin: /\$\(/,
      end: /\)/,
      contains: [ hljs.BACKSLASH_ESCAPE ]
    };
    const COMMENT = hljs.inherit(
      hljs.COMMENT(),
      {
        match: [
          /(^|\s)/,
          /#.*$/
        ],
        scope: {
          2: 'comment'
        }
      }
    );
    const HERE_DOC = {
      begin: /<<-?\s*(?=\w+)/,
      starts: { contains: [
        hljs.END_SAME_AS_BEGIN({
          begin: /(\w+)/,
          end: /(\w+)/,
          className: 'string'
        })
      ] }
    };
    const QUOTE_STRING = {
      className: 'string',
      begin: /"/,
      end: /"/,
      contains: [
        hljs.BACKSLASH_ESCAPE,
        VAR,
        SUBST
      ]
    };
    SUBST.contains.push(QUOTE_STRING);
    const ESCAPED_QUOTE = {
      match: /\\"/
    };
    const APOS_STRING = {
      className: 'string',
      begin: /'/,
      end: /'/
    };
    const ESCAPED_APOS = {
      match: /\\'/
    };
    const ARITHMETIC = {
      begin: /\$?\(\(/,
      end: /\)\)/,
      contains: [
        {
          begin: /\d+#[0-9a-f]+/,
          className: "number"
        },
        hljs.NUMBER_MODE,
        VAR
      ]
    };
    const SH_LIKE_SHELLS = [
      "fish",
      "bash",
      "zsh",
      "sh",
      "csh",
      "ksh",
      "tcsh",
      "dash",
      "scsh",
    ];
    const KNOWN_SHEBANG = hljs.SHEBANG({
      binary: `(${SH_LIKE_SHELLS.join("|")})`,
      relevance: 10
    });
    const FUNCTION = {
      className: 'function',
      begin: /\w[\w\d_]*\s*\(\s*\)\s*\{/,
      returnBegin: true,
      contains: [ hljs.inherit(hljs.TITLE_MODE, { begin: /\w[\w\d_]*/ }) ],
      relevance: 0
    };

    const KEYWORDS = [
      "if",
      "then",
      "else",
      "elif",
      "fi",
      "for",
      "while",
      "until",
      "in",
      "do",
      "done",
      "case",
      "esac",
      "function",
      "select"
    ];

    const LITERALS = [
      "true",
      "false"
    ];

    // to consume paths to prevent keyword matches inside them
    const PATH_MODE = { match: /(\/[a-z._-]+)+/ };

    // http://www.gnu.org/software/bash/manual/html_node/Shell-Builtin-Commands.html
    const SHELL_BUILT_INS = [
      "break",
      "cd",
      "continue",
      "eval",
      "exec",
      "exit",
      "export",
      "getopts",
      "hash",
      "pwd",
      "readonly",
      "return",
      "shift",
      "test",
      "times",
      "trap",
      "umask",
      "unset"
    ];

    const BASH_BUILT_INS = [
      "alias",
      "bind",
      "builtin",
      "caller",
      "command",
      "declare",
      "echo",
      "enable",
      "help",
      "let",
      "local",
      "logout",
      "mapfile",
      "printf",
      "read",
      "readarray",
      "source",
      "sudo",
      "type",
      "typeset",
      "ulimit",
      "unalias"
    ];

    const ZSH_BUILT_INS = [
      "autoload",
      "bg",
      "bindkey",
      "bye",
      "cap",
      "chdir",
      "clone",
      "comparguments",
      "compcall",
      "compctl",
      "compdescribe",
      "compfiles",
      "compgroups",
      "compquote",
      "comptags",
      "comptry",
      "compvalues",
      "dirs",
      "disable",
      "disown",
      "echotc",
      "echoti",
      "emulate",
      "fc",
      "fg",
      "float",
      "functions",
      "getcap",
      "getln",
      "history",
      "integer",
      "jobs",
      "kill",
      "limit",
      "log",
      "noglob",
      "popd",
      "print",
      "pushd",
      "pushln",
      "rehash",
      "sched",
      "setcap",
      "setopt",
      "stat",
      "suspend",
      "ttyctl",
      "unfunction",
      "unhash",
      "unlimit",
      "unsetopt",
      "vared",
      "wait",
      "whence",
      "where",
      "which",
      "zcompile",
      "zformat",
      "zftp",
      "zle",
      "zmodload",
      "zparseopts",
      "zprof",
      "zpty",
      "zregexparse",
      "zsocket",
      "zstyle",
      "ztcp"
    ];

    const GNU_CORE_UTILS = [
      "chcon",
      "chgrp",
      "chown",
      "chmod",
      "cp",
      "dd",
      "df",
      "dir",
      "dircolors",
      "ln",
      "ls",
      "mkdir",
      "mkfifo",
      "mknod",
      "mktemp",
      "mv",
      "realpath",
      "rm",
      "rmdir",
      "shred",
      "sync",
      "touch",
      "truncate",
      "vdir",
      "b2sum",
      "base32",
      "base64",
      "cat",
      "cksum",
      "comm",
      "csplit",
      "cut",
      "expand",
      "fmt",
      "fold",
      "head",
      "join",
      "md5sum",
      "nl",
      "numfmt",
      "od",
      "paste",
      "ptx",
      "pr",
      "sha1sum",
      "sha224sum",
      "sha256sum",
      "sha384sum",
      "sha512sum",
      "shuf",
      "sort",
      "split",
      "sum",
      "tac",
      "tail",
      "tr",
      "tsort",
      "unexpand",
      "uniq",
      "wc",
      "arch",
      "basename",
      "chroot",
      "date",
      "dirname",
      "du",
      "echo",
      "env",
      "expr",
      "factor",
      // "false", // keyword literal already
      "groups",
      "hostid",
      "id",
      "link",
      "logname",
      "nice",
      "nohup",
      "nproc",
      "pathchk",
      "pinky",
      "printenv",
      "printf",
      "pwd",
      "readlink",
      "runcon",
      "seq",
      "sleep",
      "stat",
      "stdbuf",
      "stty",
      "tee",
      "test",
      "timeout",
      // "true", // keyword literal already
      "tty",
      "uname",
      "unlink",
      "uptime",
      "users",
      "who",
      "whoami",
      "yes"
    ];

    return {
      name: 'Bash',
      aliases: [
        'sh',
        'zsh'
      ],
      keywords: {
        $pattern: /\b[a-z][a-z0-9._-]+\b/,
        keyword: KEYWORDS,
        literal: LITERALS,
        built_in: [
          ...SHELL_BUILT_INS,
          ...BASH_BUILT_INS,
          // Shell modifiers
          "set",
          "shopt",
          ...ZSH_BUILT_INS,
          ...GNU_CORE_UTILS
        ]
      },
      contains: [
        KNOWN_SHEBANG, // to catch known shells and boost relevancy
        hljs.SHEBANG(), // to catch unknown shells but still highlight the shebang
        FUNCTION,
        ARITHMETIC,
        COMMENT,
        HERE_DOC,
        PATH_MODE,
        QUOTE_STRING,
        ESCAPED_QUOTE,
        APOS_STRING,
        ESCAPED_APOS,
        VAR
      ]
    };
  }

  return bash;

})();

    hljs.registerLanguage('bash', hljsGrammar);
  })();/*! `cpp` grammar compiled for Highlight.js 11.10.0 */
  (function(){
    var hljsGrammar = (function () {
  'use strict';

  /*
  Language: C++
  Category: common, system
  Website: https://isocpp.org
  */

  /** @type LanguageFn */
  function cpp(hljs) {
    const regex = hljs.regex;
    // added for historic reasons because `hljs.C_LINE_COMMENT_MODE` does
    // not include such support nor can we be sure all the grammars depending
    // on it would desire this behavior
    const C_LINE_COMMENT_MODE = hljs.COMMENT('//', '$', { contains: [ { begin: /\\\n/ } ] });
    const DECLTYPE_AUTO_RE = 'decltype\\(auto\\)';
    const NAMESPACE_RE = '[a-zA-Z_]\\w*::';
    const TEMPLATE_ARGUMENT_RE = '<[^<>]+>';
    const FUNCTION_TYPE_RE = '(?!struct)('
      + DECLTYPE_AUTO_RE + '|'
      + regex.optional(NAMESPACE_RE)
      + '[a-zA-Z_]\\w*' + regex.optional(TEMPLATE_ARGUMENT_RE)
    + ')';

    const CPP_PRIMITIVE_TYPES = {
      className: 'type',
      begin: '\\b[a-z\\d_]*_t\\b'
    };

    // https://en.cppreference.com/w/cpp/language/escape
    // \\ \x \xFF \u2837 \u00323747 \374
    const CHARACTER_ESCAPES = '\\\\(x[0-9A-Fa-f]{2}|u[0-9A-Fa-f]{4,8}|[0-7]{3}|\\S)';
    const STRINGS = {
      className: 'string',
      variants: [
        {
          begin: '(u8?|U|L)?"',
          end: '"',
          illegal: '\\n',
          contains: [ hljs.BACKSLASH_ESCAPE ]
        },
        {
          begin: '(u8?|U|L)?\'(' + CHARACTER_ESCAPES + '|.)',
          end: '\'',
          illegal: '.'
        },
        hljs.END_SAME_AS_BEGIN({
          begin: /(?:u8?|U|L)?R"([^()\\ ]{0,16})\(/,
          end: /\)([^()\\ ]{0,16})"/
        })
      ]
    };

    const NUMBERS = {
      className: 'number',
      variants: [
        // Floating-point literal.
        { begin:
          "[+-]?(?:" // Leading sign.
            // Decimal.
            + "(?:"
              +"[0-9](?:'?[0-9])*\\.(?:[0-9](?:'?[0-9])*)?"
              + "|\\.[0-9](?:'?[0-9])*"
            + ")(?:[Ee][+-]?[0-9](?:'?[0-9])*)?"
            + "|[0-9](?:'?[0-9])*[Ee][+-]?[0-9](?:'?[0-9])*"
            // Hexadecimal.
            + "|0[Xx](?:"
              +"[0-9A-Fa-f](?:'?[0-9A-Fa-f])*(?:\\.(?:[0-9A-Fa-f](?:'?[0-9A-Fa-f])*)?)?"
              + "|\\.[0-9A-Fa-f](?:'?[0-9A-Fa-f])*"
            + ")[Pp][+-]?[0-9](?:'?[0-9])*"
          + ")(?:" // Literal suffixes.
            + "[Ff](?:16|32|64|128)?"
            + "|(BF|bf)16"
            + "|[Ll]"
            + "|" // Literal suffix is optional.
          + ")"
        },
        // Integer literal.
        { begin:
          "[+-]?\\b(?:" // Leading sign.
            + "0[Bb][01](?:'?[01])*" // Binary.
            + "|0[Xx][0-9A-Fa-f](?:'?[0-9A-Fa-f])*" // Hexadecimal.
            + "|0(?:'?[0-7])*" // Octal or just a lone zero.
            + "|[1-9](?:'?[0-9])*" // Decimal.
          + ")(?:" // Literal suffixes.
            + "[Uu](?:LL?|ll?)"
            + "|[Uu][Zz]?"
            + "|(?:LL?|ll?)[Uu]?"
            + "|[Zz][Uu]"
            + "|" // Literal suffix is optional.
          + ")"
          // Note: there are user-defined literal suffixes too, but perhaps having the custom suffix not part of the
          // literal highlight actually makes it stand out more.
        }
      ],
      relevance: 0
    };

    const PREPROCESSOR = {
      className: 'meta',
      begin: /#\s*[a-z]+\b/,
      end: /$/,
      keywords: { keyword:
          'if else elif endif define undef warning error line '
          + 'pragma _Pragma ifdef ifndef include' },
      contains: [
        {
          begin: /\\\n/,
          relevance: 0
        },
        hljs.inherit(STRINGS, { className: 'string' }),
        {
          className: 'string',
          begin: /<.*?>/
        },
        C_LINE_COMMENT_MODE,
        hljs.C_BLOCK_COMMENT_MODE
      ]
    };

    const TITLE_MODE = {
      className: 'title',
      begin: regex.optional(NAMESPACE_RE) + hljs.IDENT_RE,
      relevance: 0
    };

    const FUNCTION_TITLE = regex.optional(NAMESPACE_RE) + hljs.IDENT_RE + '\\s*\\(';

    // https://en.cppreference.com/w/cpp/keyword
    const RESERVED_KEYWORDS = [
      'alignas',
      'alignof',
      'and',
      'and_eq',
      'asm',
      'atomic_cancel',
      'atomic_commit',
      'atomic_noexcept',
      'auto',
      'bitand',
      'bitor',
      'break',
      'case',
      'catch',
      'class',
      'co_await',
      'co_return',
      'co_yield',
      'compl',
      'concept',
      'const_cast|10',
      'consteval',
      'constexpr',
      'constinit',
      'continue',
      'decltype',
      'default',
      'delete',
      'do',
      'dynamic_cast|10',
      'else',
      'enum',
      'explicit',
      'export',
      'extern',
      'false',
      'final',
      'for',
      'friend',
      'goto',
      'if',
      'import',
      'inline',
      'module',
      'mutable',
      'namespace',
      'new',
      'noexcept',
      'not',
      'not_eq',
      'nullptr',
      'operator',
      'or',
      'or_eq',
      'override',
      'private',
      'protected',
      'public',
      'reflexpr',
      'register',
      'reinterpret_cast|10',
      'requires',
      'return',
      'sizeof',
      'static_assert',
      'static_cast|10',
      'struct',
      'switch',
      'synchronized',
      'template',
      'this',
      'thread_local',
      'throw',
      'transaction_safe',
      'transaction_safe_dynamic',
      'true',
      'try',
      'typedef',
      'typeid',
      'typename',
      'union',
      'using',
      'virtual',
      'volatile',
      'while',
      'xor',
      'xor_eq'
    ];

    // https://en.cppreference.com/w/cpp/keyword
    const RESERVED_TYPES = [
      'bool',
      'char',
      'char16_t',
      'char32_t',
      'char8_t',
      'double',
      'float',
      'int',
      'long',
      'short',
      'void',
      'wchar_t',
      'unsigned',
      'signed',
      'const',
      'static'
    ];

    const TYPE_HINTS = [
      'any',
      'auto_ptr',
      'barrier',
      'binary_semaphore',
      'bitset',
      'complex',
      'condition_variable',
      'condition_variable_any',
      'counting_semaphore',
      'deque',
      'false_type',
      'future',
      'imaginary',
      'initializer_list',
      'istringstream',
      'jthread',
      'latch',
      'lock_guard',
      'multimap',
      'multiset',
      'mutex',
      'optional',
      'ostringstream',
      'packaged_task',
      'pair',
      'promise',
      'priority_queue',
      'queue',
      'recursive_mutex',
      'recursive_timed_mutex',
      'scoped_lock',
      'set',
      'shared_future',
      'shared_lock',
      'shared_mutex',
      'shared_timed_mutex',
      'shared_ptr',
      'stack',
      'string_view',
      'stringstream',
      'timed_mutex',
      'thread',
      'true_type',
      'tuple',
      'unique_lock',
      'unique_ptr',
      'unordered_map',
      'unordered_multimap',
      'unordered_multiset',
      'unordered_set',
      'variant',
      'vector',
      'weak_ptr',
      'wstring',
      'wstring_view'
    ];

    const FUNCTION_HINTS = [
      'abort',
      'abs',
      'acos',
      'apply',
      'as_const',
      'asin',
      'atan',
      'atan2',
      'calloc',
      'ceil',
      'cerr',
      'cin',
      'clog',
      'cos',
      'cosh',
      'cout',
      'declval',
      'endl',
      'exchange',
      'exit',
      'exp',
      'fabs',
      'floor',
      'fmod',
      'forward',
      'fprintf',
      'fputs',
      'free',
      'frexp',
      'fscanf',
      'future',
      'invoke',
      'isalnum',
      'isalpha',
      'iscntrl',
      'isdigit',
      'isgraph',
      'islower',
      'isprint',
      'ispunct',
      'isspace',
      'isupper',
      'isxdigit',
      'labs',
      'launder',
      'ldexp',
      'log',
      'log10',
      'make_pair',
      'make_shared',
      'make_shared_for_overwrite',
      'make_tuple',
      'make_unique',
      'malloc',
      'memchr',
      'memcmp',
      'memcpy',
      'memset',
      'modf',
      'move',
      'pow',
      'printf',
      'putchar',
      'puts',
      'realloc',
      'scanf',
      'sin',
      'sinh',
      'snprintf',
      'sprintf',
      'sqrt',
      'sscanf',
      'std',
      'stderr',
      'stdin',
      'stdout',
      'strcat',
      'strchr',
      'strcmp',
      'strcpy',
      'strcspn',
      'strlen',
      'strncat',
      'strncmp',
      'strncpy',
      'strpbrk',
      'strrchr',
      'strspn',
      'strstr',
      'swap',
      'tan',
      'tanh',
      'terminate',
      'to_underlying',
      'tolower',
      'toupper',
      'vfprintf',
      'visit',
      'vprintf',
      'vsprintf'
    ];

    const LITERALS = [
      'NULL',
      'false',
      'nullopt',
      'nullptr',
      'true'
    ];

    // https://en.cppreference.com/w/cpp/keyword
    const BUILT_IN = [ '_Pragma' ];

    const CPP_KEYWORDS = {
      type: RESERVED_TYPES,
      keyword: RESERVED_KEYWORDS,
      literal: LITERALS,
      built_in: BUILT_IN,
      _type_hints: TYPE_HINTS
    };

    const FUNCTION_DISPATCH = {
      className: 'function.dispatch',
      relevance: 0,
      keywords: {
        // Only for relevance, not highlighting.
        _hint: FUNCTION_HINTS },
      begin: regex.concat(
        /\b/,
        /(?!decltype)/,
        /(?!if)/,
        /(?!for)/,
        /(?!switch)/,
        /(?!while)/,
        hljs.IDENT_RE,
        regex.lookahead(/(<[^<>]+>|)\s*\(/))
    };

    const EXPRESSION_CONTAINS = [
      FUNCTION_DISPATCH,
      PREPROCESSOR,
      CPP_PRIMITIVE_TYPES,
      C_LINE_COMMENT_MODE,
      hljs.C_BLOCK_COMMENT_MODE,
      NUMBERS,
      STRINGS
    ];

    const EXPRESSION_CONTEXT = {
      // This mode covers expression context where we can't expect a function
      // definition and shouldn't highlight anything that looks like one:
      // `return some()`, `else if()`, `(x*sum(1, 2))`
      variants: [
        {
          begin: /=/,
          end: /;/
        },
        {
          begin: /\(/,
          end: /\)/
        },
        {
          beginKeywords: 'new throw return else',
          end: /;/
        }
      ],
      keywords: CPP_KEYWORDS,
      contains: EXPRESSION_CONTAINS.concat([
        {
          begin: /\(/,
          end: /\)/,
          keywords: CPP_KEYWORDS,
          contains: EXPRESSION_CONTAINS.concat([ 'self' ]),
          relevance: 0
        }
      ]),
      relevance: 0
    };

    const FUNCTION_DECLARATION = {
      className: 'function',
      begin: '(' + FUNCTION_TYPE_RE + '[\\*&\\s]+)+' + FUNCTION_TITLE,
      returnBegin: true,
      end: /[{;=]/,
      excludeEnd: true,
      keywords: CPP_KEYWORDS,
      illegal: /[^\w\s\*&:<>.]/,
      contains: [
        { // to prevent it from being confused as the function title
          begin: DECLTYPE_AUTO_RE,
          keywords: CPP_KEYWORDS,
          relevance: 0
        },
        {
          begin: FUNCTION_TITLE,
          returnBegin: true,
          contains: [ TITLE_MODE ],
          relevance: 0
        },
        // needed because we do not have look-behind on the below rule
        // to prevent it from grabbing the final : in a :: pair
        {
          begin: /::/,
          relevance: 0
        },
        // initializers
        {
          begin: /:/,
          endsWithParent: true,
          contains: [
            STRINGS,
            NUMBERS
          ]
        },
        // allow for multiple declarations, e.g.:
        // extern void f(int), g(char);
        {
          relevance: 0,
          match: /,/
        },
        {
          className: 'params',
          begin: /\(/,
          end: /\)/,
          keywords: CPP_KEYWORDS,
          relevance: 0,
          contains: [
            C_LINE_COMMENT_MODE,
            hljs.C_BLOCK_COMMENT_MODE,
            STRINGS,
            NUMBERS,
            CPP_PRIMITIVE_TYPES,
            // Count matching parentheses.
            {
              begin: /\(/,
              end: /\)/,
              keywords: CPP_KEYWORDS,
              relevance: 0,
              contains: [
                'self',
                C_LINE_COMMENT_MODE,
                hljs.C_BLOCK_COMMENT_MODE,
                STRINGS,
                NUMBERS,
                CPP_PRIMITIVE_TYPES
              ]
            }
          ]
        },
        CPP_PRIMITIVE_TYPES,
        C_LINE_COMMENT_MODE,
        hljs.C_BLOCK_COMMENT_MODE,
        PREPROCESSOR
      ]
    };

    return {
      name: 'C++',
      aliases: [
        'cc',
        'c++',
        'h++',
        'hpp',
        'hh',
        'hxx',
        'cxx'
      ],
      keywords: CPP_KEYWORDS,
      illegal: '</',
      classNameAliases: { 'function.dispatch': 'built_in' },
      contains: [].concat(
        EXPRESSION_CONTEXT,
        FUNCTION_DECLARATION,
        FUNCTION_DISPATCH,
        EXPRESSION_CONTAINS,
        [
          PREPROCESSOR,
          { // containers: ie, `vector <int> rooms (9);`
            begin: '\\b(deque|list|queue|priority_queue|pair|stack|vector|map|set|bitset|multiset|multimap|unordered_map|unordered_set|unordered_multiset|unordered_multimap|array|tuple|optional|variant|function)\\s*<(?!<)',
            end: '>',
            keywords: CPP_KEYWORDS,
            contains: [
              'self',
              CPP_PRIMITIVE_TYPES
            ]
          },
          {
            begin: hljs.IDENT_RE + '::',
            keywords: CPP_KEYWORDS
          },
          {
            match: [
              // extra complexity to deal with `enum class` and `enum struct`
              /\b(?:enum(?:\s+(?:class|struct))?|class|struct|union)/,
              /\s+/,
              /\w+/
            ],
            className: {
              1: 'keyword',
              3: 'title.class'
            }
          }
        ])
    };
  }

  return cpp;

})();

    hljs.registerLanguage('cpp', hljsGrammar);
  })();/*! `ini` grammar compiled for Highlight.js 11.10.0 */
  (function(){
    var hljsGrammar = (function () {
  'use strict';

  /*
  Language: TOML, also INI
  Description: TOML aims to be a minimal configuration file format that's easy to read due to obvious semantics.
  Contributors: Guillaume Gomez <guillaume1.gomez@gmail.com>
  Category: common, config
  Website: https://github.com/toml-lang/toml
  */

  function ini(hljs) {
    const regex = hljs.regex;
    const NUMBERS = {
      className: 'number',
      relevance: 0,
      variants: [
        { begin: /([+-]+)?[\d]+_[\d_]+/ },
        { begin: hljs.NUMBER_RE }
      ]
    };
    const COMMENTS = hljs.COMMENT();
    COMMENTS.variants = [
      {
        begin: /;/,
        end: /$/
      },
      {
        begin: /#/,
        end: /$/
      }
    ];
    const VARIABLES = {
      className: 'variable',
      variants: [
        { begin: /\$[\w\d"][\w\d_]*/ },
        { begin: /\$\{(.*?)\}/ }
      ]
    };
    const LITERALS = {
      className: 'literal',
      begin: /\bon|off|true|false|yes|no\b/
    };
    const STRINGS = {
      className: "string",
      contains: [ hljs.BACKSLASH_ESCAPE ],
      variants: [
        {
          begin: "'''",
          end: "'''",
          relevance: 10
        },
        {
          begin: '"""',
          end: '"""',
          relevance: 10
        },
        {
          begin: '"',
          end: '"'
        },
        {
          begin: "'",
          end: "'"
        }
      ]
    };
    const ARRAY = {
      begin: /\[/,
      end: /\]/,
      contains: [
        COMMENTS,
        LITERALS,
        VARIABLES,
        STRINGS,
        NUMBERS,
        'self'
      ],
      relevance: 0
    };

    const BARE_KEY = /[A-Za-z0-9_-]+/;
    const QUOTED_KEY_DOUBLE_QUOTE = /"(\\"|[^"])*"/;
    const QUOTED_KEY_SINGLE_QUOTE = /'[^']*'/;
    const ANY_KEY = regex.either(
      BARE_KEY, QUOTED_KEY_DOUBLE_QUOTE, QUOTED_KEY_SINGLE_QUOTE
    );
    const DOTTED_KEY = regex.concat(
      ANY_KEY, '(\\s*\\.\\s*', ANY_KEY, ')*',
      regex.lookahead(/\s*=\s*[^#\s]/)
    );

    return {
      name: 'TOML, also INI',
      aliases: [ 'toml' ],
      case_insensitive: true,
      illegal: /\S/,
      contains: [
        COMMENTS,
        {
          className: 'section',
          begin: /\[+/,
          end: /\]+/
        },
        {
          begin: DOTTED_KEY,
          className: 'attr',
          starts: {
            end: /$/,
            contains: [
              COMMENTS,
              ARRAY,
              LITERALS,
              VARIABLES,
              STRINGS,
              NUMBERS
            ]
          }
        }
      ]
    };
  }

  return ini;

})();

    hljs.registerLanguage('ini', hljsGrammar);
  })();/*! `javascript` grammar compiled for Highlight.js 11.10.0 */
  (function(){
    var hljsGrammar = (function () {
  'use strict';

  const IDENT_RE = '[A-Za-z$_][0-9A-Za-z$_]*';
  const KEYWORDS = [
    "as", // for exports
    "in",
    "of",
    "if",
    "for",
    "while",
    "finally",
    "var",
    "new",
    "function",
    "do",
    "return",
    "void",
    "else",
    "break",
    "catch",
    "instanceof",
    "with",
    "throw",
    "case",
    "default",
    "try",
    "switch",
    "continue",
    "typeof",
    "delete",
    "let",
    "yield",
    "const",
    "class",
    // JS handles these with a special rule
    // "get",
    // "set",
    "debugger",
    "async",
    "await",
    "static",
    "import",
    "from",
    "export",
    "extends"
  ];
  const LITERALS = [
    "true",
    "false",
    "null",
    "undefined",
    "NaN",
    "Infinity"
  ];

  // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects
  const TYPES = [
    // Fundamental objects
    "Object",
    "Function",
    "Boolean",
    "Symbol",
    // numbers and dates
    "Math",
    "Date",
    "Number",
    "BigInt",
    // text
    "String",
    "RegExp",
    // Indexed collections
    "Array",
    "Float32Array",
    "Float64Array",
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Int32Array",
    "Uint16Array",
    "Uint32Array",
    "BigInt64Array",
    "BigUint64Array",
    // Keyed collections
    "Set",
    "Map",
    "WeakSet",
    "WeakMap",
    // Structured data
    "ArrayBuffer",
    "SharedArrayBuffer",
    "Atomics",
    "DataView",
    "JSON",
    // Control abstraction objects
    "Promise",
    "Generator",
    "GeneratorFunction",
    "AsyncFunction",
    // Reflection
    "Reflect",
    "Proxy",
    // Internationalization
    "Intl",
    // WebAssembly
    "WebAssembly"
  ];

  const ERROR_TYPES = [
    "Error",
    "EvalError",
    "InternalError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "TypeError",
    "URIError"
  ];

  const BUILT_IN_GLOBALS = [
    "setInterval",
    "setTimeout",
    "clearInterval",
    "clearTimeout",

    "require",
    "exports",

    "eval",
    "isFinite",
    "isNaN",
    "parseFloat",
    "parseInt",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
    "escape",
    "unescape"
  ];

  const BUILT_IN_VARIABLES = [
    "arguments",
    "this",
    "super",
    "console",
    "window",
    "document",
    "localStorage",
    "sessionStorage",
    "module",
    "global" // Node.js
  ];

  const BUILT_INS = [].concat(
    BUILT_IN_GLOBALS,
    TYPES,
    ERROR_TYPES
  );

  /*
  Language: JavaScript
  Description: JavaScript (JS) is a lightweight, interpreted, or just-in-time compiled programming language with first-class functions.
  Category: common, scripting, web
  Website: https://developer.mozilla.org/en-US/docs/Web/JavaScript
  */


  /** @type LanguageFn */
  function javascript(hljs) {
    const regex = hljs.regex;
    /**
     * Takes a string like "<Booger" and checks to see
     * if we can find a matching "</Booger" later in the
     * content.
     * @param {RegExpMatchArray} match
     * @param {{after:number}} param1
     */
    const hasClosingTag = (match, { after }) => {
      const tag = "</" + match[0].slice(1);
      const pos = match.input.indexOf(tag, after);
      return pos !== -1;
    };

    const IDENT_RE$1 = IDENT_RE;
    const FRAGMENT = {
      begin: '<>',
      end: '</>'
    };
    // to avoid some special cases inside isTrulyOpeningTag
    const XML_SELF_CLOSING = /<[A-Za-z0-9\\._:-]+\s*\/>/;
    const XML_TAG = {
      begin: /<[A-Za-z0-9\\._:-]+/,
      end: /\/[A-Za-z0-9\\._:-]+>|\/>/,
      /**
       * @param {RegExpMatchArray} match
       * @param {CallbackResponse} response
       */
      isTrulyOpeningTag: (match, response) => {
        const afterMatchIndex = match[0].length + match.index;
        const nextChar = match.input[afterMatchIndex];
        if (
          // HTML should not include another raw `<` inside a tag
          // nested type?
          // `<Array<Array<number>>`, etc.
          nextChar === "<" ||
          // the , gives away that this is not HTML
          // `<T, A extends keyof T, V>`
          nextChar === ","
          ) {
          response.ignoreMatch();
          return;
        }

        // `<something>`
        // Quite possibly a tag, lets look for a matching closing tag...
        if (nextChar === ">") {
          // if we cannot find a matching closing tag, then we
          // will ignore it
          if (!hasClosingTag(match, { after: afterMatchIndex })) {
            response.ignoreMatch();
          }
        }

        // `<blah />` (self-closing)
        // handled by simpleSelfClosing rule

        let m;
        const afterMatch = match.input.substring(afterMatchIndex);

        // some more template typing stuff
        //  <T = any>(key?: string) => Modify<
        if ((m = afterMatch.match(/^\s*=/))) {
          response.ignoreMatch();
          return;
        }

        // `<From extends string>`
        // technically this could be HTML, but it smells like a type
        // NOTE: This is ugh, but added specifically for https://github.com/highlightjs/highlight.js/issues/3276
        if ((m = afterMatch.match(/^\s+extends\s+/))) {
          if (m.index === 0) {
            response.ignoreMatch();
            // eslint-disable-next-line no-useless-return
            return;
          }
        }
      }
    };
    const KEYWORDS$1 = {
      $pattern: IDENT_RE,
      keyword: KEYWORDS,
      literal: LITERALS,
      built_in: BUILT_INS,
      "variable.language": BUILT_IN_VARIABLES
    };

    // https://tc39.es/ecma262/#sec-literals-numeric-literals
    const decimalDigits = '[0-9](_?[0-9])*';
    const frac = `\\.(${decimalDigits})`;
    // DecimalIntegerLiteral, including Annex B NonOctalDecimalIntegerLiteral
    // https://tc39.es/ecma262/#sec-additional-syntax-numeric-literals
    const decimalInteger = `0|[1-9](_?[0-9])*|0[0-7]*[89][0-9]*`;
    const NUMBER = {
      className: 'number',
      variants: [
        // DecimalLiteral
        { begin: `(\\b(${decimalInteger})((${frac})|\\.)?|(${frac}))` +
          `[eE][+-]?(${decimalDigits})\\b` },
        { begin: `\\b(${decimalInteger})\\b((${frac})\\b|\\.)?|(${frac})\\b` },

        // DecimalBigIntegerLiteral
        { begin: `\\b(0|[1-9](_?[0-9])*)n\\b` },

        // NonDecimalIntegerLiteral
        { begin: "\\b0[xX][0-9a-fA-F](_?[0-9a-fA-F])*n?\\b" },
        { begin: "\\b0[bB][0-1](_?[0-1])*n?\\b" },
        { begin: "\\b0[oO][0-7](_?[0-7])*n?\\b" },

        // LegacyOctalIntegerLiteral (does not include underscore separators)
        // https://tc39.es/ecma262/#sec-additional-syntax-numeric-literals
        { begin: "\\b0[0-7]+n?\\b" },
      ],
      relevance: 0
    };

    const SUBST = {
      className: 'subst',
      begin: '\\$\\{',
      end: '\\}',
      keywords: KEYWORDS$1,
      contains: [] // defined later
    };
    const HTML_TEMPLATE = {
      begin: '\.?html`',
      end: '',
      starts: {
        end: '`',
        returnEnd: false,
        contains: [
          hljs.BACKSLASH_ESCAPE,
          SUBST
        ],
        subLanguage: 'xml'
      }
    };
    const CSS_TEMPLATE = {
      begin: '\.?css`',
      end: '',
      starts: {
        end: '`',
        returnEnd: false,
        contains: [
          hljs.BACKSLASH_ESCAPE,
          SUBST
        ],
        subLanguage: 'css'
      }
    };
    const GRAPHQL_TEMPLATE = {
      begin: '\.?gql`',
      end: '',
      starts: {
        end: '`',
        returnEnd: false,
        contains: [
          hljs.BACKSLASH_ESCAPE,
          SUBST
        ],
        subLanguage: 'graphql'
      }
    };
    const TEMPLATE_STRING = {
      className: 'string',
      begin: '`',
      end: '`',
      contains: [
        hljs.BACKSLASH_ESCAPE,
        SUBST
      ]
    };
    const JSDOC_COMMENT = hljs.COMMENT(
      /\/\*\*(?!\/)/,
      '\\*/',
      {
        relevance: 0,
        contains: [
          {
            begin: '(?=@[A-Za-z]+)',
            relevance: 0,
            contains: [
              {
                className: 'doctag',
                begin: '@[A-Za-z]+'
              },
              {
                className: 'type',
                begin: '\\{',
                end: '\\}',
                excludeEnd: true,
                excludeBegin: true,
                relevance: 0
              },
              {
                className: 'variable',
                begin: IDENT_RE$1 + '(?=\\s*(-)|$)',
                endsParent: true,
                relevance: 0
              },
              // eat spaces (not newlines) so we can find
              // types or variables
              {
                begin: /(?=[^\n])\s/,
                relevance: 0
              }
            ]
          }
        ]
      }
    );
    const COMMENT = {
      className: "comment",
      variants: [
        JSDOC_COMMENT,
        hljs.C_BLOCK_COMMENT_MODE,
        hljs.C_LINE_COMMENT_MODE
      ]
    };
    const SUBST_INTERNALS = [
      hljs.APOS_STRING_MODE,
      hljs.QUOTE_STRING_MODE,
      HTML_TEMPLATE,
      CSS_TEMPLATE,
      GRAPHQL_TEMPLATE,
      TEMPLATE_STRING,
      // Skip numbers when they are part of a variable name
      { match: /\$\d+/ },
      NUMBER,
      // This is intentional:
      // See https://github.com/highlightjs/highlight.js/issues/3288
      // hljs.REGEXP_MODE
    ];
    SUBST.contains = SUBST_INTERNALS
      .concat({
        // we need to pair up {} inside our subst to prevent
        // it from ending too early by matching another }
        begin: /\{/,
        end: /\}/,
        keywords: KEYWORDS$1,
        contains: [
          "self"
        ].concat(SUBST_INTERNALS)
      });
    const SUBST_AND_COMMENTS = [].concat(COMMENT, SUBST.contains);
    const PARAMS_CONTAINS = SUBST_AND_COMMENTS.concat([
      // eat recursive parens in sub expressions
      {
        begin: /(\s*)\(/,
        end: /\)/,
        keywords: KEYWORDS$1,
        contains: ["self"].concat(SUBST_AND_COMMENTS)
      }
    ]);
    const PARAMS = {
      className: 'params',
      // convert this to negative lookbehind in v12
      begin: /(\s*)\(/, // to match the parms with 
      end: /\)/,
      excludeBegin: true,
      excludeEnd: true,
      keywords: KEYWORDS$1,
      contains: PARAMS_CONTAINS
    };

    // ES6 classes
    const CLASS_OR_EXTENDS = {
      variants: [
        // class Car extends vehicle
        {
          match: [
            /class/,
            /\s+/,
            IDENT_RE$1,
            /\s+/,
            /extends/,
            /\s+/,
            regex.concat(IDENT_RE$1, "(", regex.concat(/\./, IDENT_RE$1), ")*")
          ],
          scope: {
            1: "keyword",
            3: "title.class",
            5: "keyword",
            7: "title.class.inherited"
          }
        },
        // class Car
        {
          match: [
            /class/,
            /\s+/,
            IDENT_RE$1
          ],
          scope: {
            1: "keyword",
            3: "title.class"
          }
        },

      ]
    };

    const CLASS_REFERENCE = {
      relevance: 0,
      match:
      regex.either(
        // Hard coded exceptions
        /\bJSON/,
        // Float32Array, OutT
        /\b[A-Z][a-z]+([A-Z][a-z]*|\d)*/,
        // CSSFactory, CSSFactoryT
        /\b[A-Z]{2,}([A-Z][a-z]+|\d)+([A-Z][a-z]*)*/,
        // FPs, FPsT
        /\b[A-Z]{2,}[a-z]+([A-Z][a-z]+|\d)*([A-Z][a-z]*)*/,
        // P
        // single letters are not highlighted
        // BLAH
        // this will be flagged as a UPPER_CASE_CONSTANT instead
      ),
      className: "title.class",
      keywords: {
        _: [
          // se we still get relevance credit for JS library classes
          ...TYPES,
          ...ERROR_TYPES
        ]
      }
    };

    const USE_STRICT = {
      label: "use_strict",
      className: 'meta',
      relevance: 10,
      begin: /^\s*['"]use (strict|asm)['"]/
    };

    const FUNCTION_DEFINITION = {
      variants: [
        {
          match: [
            /function/,
            /\s+/,
            IDENT_RE$1,
            /(?=\s*\()/
          ]
        },
        // anonymous function
        {
          match: [
            /function/,
            /\s*(?=\()/
          ]
        }
      ],
      className: {
        1: "keyword",
        3: "title.function"
      },
      label: "func.def",
      contains: [ PARAMS ],
      illegal: /%/
    };

    const UPPER_CASE_CONSTANT = {
      relevance: 0,
      match: /\b[A-Z][A-Z_0-9]+\b/,
      className: "variable.constant"
    };

    function noneOf(list) {
      return regex.concat("(?!", list.join("|"), ")");
    }

    const FUNCTION_CALL = {
      match: regex.concat(
        /\b/,
        noneOf([
          ...BUILT_IN_GLOBALS,
          "super",
          "import"
        ].map(x => `${x}\\s*\\(`)),
        IDENT_RE$1, regex.lookahead(/\s*\(/)),
      className: "title.function",
      relevance: 0
    };

    const PROPERTY_ACCESS = {
      begin: regex.concat(/\./, regex.lookahead(
        regex.concat(IDENT_RE$1, /(?![0-9A-Za-z$_(])/)
      )),
      end: IDENT_RE$1,
      excludeBegin: true,
      keywords: "prototype",
      className: "property",
      relevance: 0
    };

    const GETTER_OR_SETTER = {
      match: [
        /get|set/,
        /\s+/,
        IDENT_RE$1,
        /(?=\()/
      ],
      className: {
        1: "keyword",
        3: "title.function"
      },
      contains: [
        { // eat to avoid empty params
          begin: /\(\)/
        },
        PARAMS
      ]
    };

    const FUNC_LEAD_IN_RE = '(\\(' +
      '[^()]*(\\(' +
      '[^()]*(\\(' +
      '[^()]*' +
      '\\)[^()]*)*' +
      '\\)[^()]*)*' +
      '\\)|' + hljs.UNDERSCORE_IDENT_RE + ')\\s*=>';

    const FUNCTION_VARIABLE = {
      match: [
        /const|var|let/, /\s+/,
        IDENT_RE$1, /\s*/,
        /=\s*/,
        /(async\s*)?/, // async is optional
        regex.lookahead(FUNC_LEAD_IN_RE)
      ],
      keywords: "async",
      className: {
        1: "keyword",
        3: "title.function"
      },
      contains: [
        PARAMS
      ]
    };

    return {
      name: 'JavaScript',
      aliases: ['js', 'jsx', 'mjs', 'cjs'],
      keywords: KEYWORDS$1,
      // this will be extended by TypeScript
      exports: { PARAMS_CONTAINS, CLASS_REFERENCE },
      illegal: /#(?![$_A-z])/,
      contains: [
        hljs.SHEBANG({
          label: "shebang",
          binary: "node",
          relevance: 5
        }),
        USE_STRICT,
        hljs.APOS_STRING_MODE,
        hljs.QUOTE_STRING_MODE,
        HTML_TEMPLATE,
        CSS_TEMPLATE,
        GRAPHQL_TEMPLATE,
        TEMPLATE_STRING,
        COMMENT,
        // Skip numbers when they are part of a variable name
        { match: /\$\d+/ },
        NUMBER,
        CLASS_REFERENCE,
        {
          className: 'attr',
          begin: IDENT_RE$1 + regex.lookahead(':'),
          relevance: 0
        },
        FUNCTION_VARIABLE,
        { // "value" container
          begin: '(' + hljs.RE_STARTERS_RE + '|\\b(case|return|throw)\\b)\\s*',
          keywords: 'return throw case',
          relevance: 0,
          contains: [
            COMMENT,
            hljs.REGEXP_MODE,
            {
              className: 'function',
              // we have to count the parens to make sure we actually have the
              // correct bounding ( ) before the =>.  There could be any number of
              // sub-expressions inside also surrounded by parens.
              begin: FUNC_LEAD_IN_RE,
              returnBegin: true,
              end: '\\s*=>',
              contains: [
                {
                  className: 'params',
                  variants: [
                    {
                      begin: hljs.UNDERSCORE_IDENT_RE,
                      relevance: 0
                    },
                    {
                      className: null,
                      begin: /\(\s*\)/,
                      skip: true
                    },
                    {
                      begin: /(\s*)\(/,
                      end: /\)/,
                      excludeBegin: true,
                      excludeEnd: true,
                      keywords: KEYWORDS$1,
                      contains: PARAMS_CONTAINS
                    }
                  ]
                }
              ]
            },
            { // could be a comma delimited list of params to a function call
              begin: /,/,
              relevance: 0
            },
            {
              match: /\s+/,
              relevance: 0
            },
            { // JSX
              variants: [
                { begin: FRAGMENT.begin, end: FRAGMENT.end },
                { match: XML_SELF_CLOSING },
                {
                  begin: XML_TAG.begin,
                  // we carefully check the opening tag to see if it truly
                  // is a tag and not a false positive
                  'on:begin': XML_TAG.isTrulyOpeningTag,
                  end: XML_TAG.end
                }
              ],
              subLanguage: 'xml',
              contains: [
                {
                  begin: XML_TAG.begin,
                  end: XML_TAG.end,
                  skip: true,
                  contains: ['self']
                }
              ]
            }
          ],
        },
        FUNCTION_DEFINITION,
        {
          // prevent this from getting swallowed up by function
          // since they appear "function like"
          beginKeywords: "while if switch catch for"
        },
        {
          // we have to count the parens to make sure we actually have the correct
          // bounding ( ).  There could be any number of sub-expressions inside
          // also surrounded by parens.
          begin: '\\b(?!function)' + hljs.UNDERSCORE_IDENT_RE +
            '\\(' + // first parens
            '[^()]*(\\(' +
              '[^()]*(\\(' +
                '[^()]*' +
              '\\)[^()]*)*' +
            '\\)[^()]*)*' +
            '\\)\\s*\\{', // end parens
          returnBegin:true,
          label: "func.def",
          contains: [
            PARAMS,
            hljs.inherit(hljs.TITLE_MODE, { begin: IDENT_RE$1, className: "title.function" })
          ]
        },
        // catch ... so it won't trigger the property rule below
        {
          match: /\.\.\./,
          relevance: 0
        },
        PROPERTY_ACCESS,
        // hack: prevents detection of keywords in some circumstances
        // .keyword()
        // $keyword = x
        {
          match: '\\$' + IDENT_RE$1,
          relevance: 0
        },
        {
          match: [ /\bconstructor(?=\s*\()/ ],
          className: { 1: "title.function" },
          contains: [ PARAMS ]
        },
        FUNCTION_CALL,
        UPPER_CASE_CONSTANT,
        CLASS_OR_EXTENDS,
        GETTER_OR_SETTER,
        {
          match: /\$[(.]/ // relevance booster for a pattern common to JS libs: `$(something)` and `$.something`
        }
      ]
    };
  }

  return javascript;

})();

    hljs.registerLanguage('javascript', hljsGrammar);
  })();/*! `plaintext` grammar compiled for Highlight.js 11.10.0 */
  (function(){
    var hljsGrammar = (function () {
  'use strict';

  /*
  Language: Plain text
  Author: Egor Rogov (e.rogov@postgrespro.ru)
  Description: Plain text without any highlighting.
  Category: common
  */

  function plaintext(hljs) {
    return {
      name: 'Plain text',
      aliases: [
        'text',
        'txt'
      ],
      disableAutodetect: true
    };
  }

  return plaintext;

})();

    hljs.registerLanguage('plaintext', hljsGrammar);
  })();/*! `powershell` grammar compiled for Highlight.js 11.10.0 */
  (function(){
    var hljsGrammar = (function () {
  'use strict';

  /*
  Language: PowerShell
  Description: PowerShell is a task-based command-line shell and scripting language built on .NET.
  Author: David Mohundro <david@mohundro.com>
  Contributors: Nicholas Blumhardt <nblumhardt@nblumhardt.com>, Victor Zhou <OiCMudkips@users.noreply.github.com>, Nicolas Le Gall <contact@nlegall.fr>
  Website: https://docs.microsoft.com/en-us/powershell/
  Category: scripting
  */

  function powershell(hljs) {
    const TYPES = [
      "string",
      "char",
      "byte",
      "int",
      "long",
      "bool",
      "decimal",
      "single",
      "double",
      "DateTime",
      "xml",
      "array",
      "hashtable",
      "void"
    ];

    // https://docs.microsoft.com/en-us/powershell/scripting/developer/cmdlet/approved-verbs-for-windows-powershell-commands
    const VALID_VERBS =
      'Add|Clear|Close|Copy|Enter|Exit|Find|Format|Get|Hide|Join|Lock|'
      + 'Move|New|Open|Optimize|Pop|Push|Redo|Remove|Rename|Reset|Resize|'
      + 'Search|Select|Set|Show|Skip|Split|Step|Switch|Undo|Unlock|'
      + 'Watch|Backup|Checkpoint|Compare|Compress|Convert|ConvertFrom|'
      + 'ConvertTo|Dismount|Edit|Expand|Export|Group|Import|Initialize|'
      + 'Limit|Merge|Mount|Out|Publish|Restore|Save|Sync|Unpublish|Update|'
      + 'Approve|Assert|Build|Complete|Confirm|Deny|Deploy|Disable|Enable|Install|Invoke|'
      + 'Register|Request|Restart|Resume|Start|Stop|Submit|Suspend|Uninstall|'
      + 'Unregister|Wait|Debug|Measure|Ping|Repair|Resolve|Test|Trace|Connect|'
      + 'Disconnect|Read|Receive|Send|Write|Block|Grant|Protect|Revoke|Unblock|'
      + 'Unprotect|Use|ForEach|Sort|Tee|Where';

    const COMPARISON_OPERATORS =
      '-and|-as|-band|-bnot|-bor|-bxor|-casesensitive|-ccontains|-ceq|-cge|-cgt|'
      + '-cle|-clike|-clt|-cmatch|-cne|-cnotcontains|-cnotlike|-cnotmatch|-contains|'
      + '-creplace|-csplit|-eq|-exact|-f|-file|-ge|-gt|-icontains|-ieq|-ige|-igt|'
      + '-ile|-ilike|-ilt|-imatch|-in|-ine|-inotcontains|-inotlike|-inotmatch|'
      + '-ireplace|-is|-isnot|-isplit|-join|-le|-like|-lt|-match|-ne|-not|'
      + '-notcontains|-notin|-notlike|-notmatch|-or|-regex|-replace|-shl|-shr|'
      + '-split|-wildcard|-xor';

    const KEYWORDS = {
      $pattern: /-?[A-z\.\-]+\b/,
      keyword:
        'if else foreach return do while until elseif begin for trap data dynamicparam '
        + 'end break throw param continue finally in switch exit filter try process catch '
        + 'hidden static parameter',
      // "echo" relevance has been set to 0 to avoid auto-detect conflicts with shell transcripts
      built_in:
        'ac asnp cat cd CFS chdir clc clear clhy cli clp cls clv cnsn compare copy cp '
        + 'cpi cpp curl cvpa dbp del diff dir dnsn ebp echo|0 epal epcsv epsn erase etsn exsn fc fhx '
        + 'fl ft fw gal gbp gc gcb gci gcm gcs gdr gerr ghy gi gin gjb gl gm gmo gp gps gpv group '
        + 'gsn gsnp gsv gtz gu gv gwmi h history icm iex ihy ii ipal ipcsv ipmo ipsn irm ise iwmi '
        + 'iwr kill lp ls man md measure mi mount move mp mv nal ndr ni nmo npssc nsn nv ogv oh '
        + 'popd ps pushd pwd r rbp rcjb rcsn rd rdr ren ri rjb rm rmdir rmo rni rnp rp rsn rsnp '
        + 'rujb rv rvpa rwmi sajb sal saps sasv sbp sc scb select set shcm si sl sleep sls sort sp '
        + 'spjb spps spsv start stz sujb sv swmi tee trcm type wget where wjb write'
      // TODO: 'validate[A-Z]+' can't work in keywords
    };

    const TITLE_NAME_RE = /\w[\w\d]*((-)[\w\d]+)*/;

    const BACKTICK_ESCAPE = {
      begin: '`[\\s\\S]',
      relevance: 0
    };

    const VAR = {
      className: 'variable',
      variants: [
        { begin: /\$\B/ },
        {
          className: 'keyword',
          begin: /\$this/
        },
        { begin: /\$[\w\d][\w\d_:]*/ }
      ]
    };

    const LITERAL = {
      className: 'literal',
      begin: /\$(null|true|false)\b/
    };

    const QUOTE_STRING = {
      className: "string",
      variants: [
        {
          begin: /"/,
          end: /"/
        },
        {
          begin: /@"/,
          end: /^"@/
        }
      ],
      contains: [
        BACKTICK_ESCAPE,
        VAR,
        {
          className: 'variable',
          begin: /\$[A-z]/,
          end: /[^A-z]/
        }
      ]
    };

    const APOS_STRING = {
      className: 'string',
      variants: [
        {
          begin: /'/,
          end: /'/
        },
        {
          begin: /@'/,
          end: /^'@/
        }
      ]
    };

    const PS_HELPTAGS = {
      className: "doctag",
      variants: [
        /* no paramater help tags */
        { begin: /\.(synopsis|description|example|inputs|outputs|notes|link|component|role|functionality)/ },
        /* one parameter help tags */
        { begin: /\.(parameter|forwardhelptargetname|forwardhelpcategory|remotehelprunspace|externalhelp)\s+\S+/ }
      ]
    };

    const PS_COMMENT = hljs.inherit(
      hljs.COMMENT(null, null),
      {
        variants: [
          /* single-line comment */
          {
            begin: /#/,
            end: /$/
          },
          /* multi-line comment */
          {
            begin: /<#/,
            end: /#>/
          }
        ],
        contains: [ PS_HELPTAGS ]
      }
    );

    const CMDLETS = {
      className: 'built_in',
      variants: [ { begin: '('.concat(VALID_VERBS, ')+(-)[\\w\\d]+') } ]
    };

    const PS_CLASS = {
      className: 'class',
      beginKeywords: 'class enum',
      end: /\s*[{]/,
      excludeEnd: true,
      relevance: 0,
      contains: [ hljs.TITLE_MODE ]
    };

    const PS_FUNCTION = {
      className: 'function',
      begin: /function\s+/,
      end: /\s*\{|$/,
      excludeEnd: true,
      returnBegin: true,
      relevance: 0,
      contains: [
        {
          begin: "function",
          relevance: 0,
          className: "keyword"
        },
        {
          className: "title",
          begin: TITLE_NAME_RE,
          relevance: 0
        },
        {
          begin: /\(/,
          end: /\)/,
          className: "params",
          relevance: 0,
          contains: [ VAR ]
        }
        // CMDLETS
      ]
    };

    // Using statment, plus type, plus assembly name.
    const PS_USING = {
      begin: /using\s/,
      end: /$/,
      returnBegin: true,
      contains: [
        QUOTE_STRING,
        APOS_STRING,
        {
          className: 'keyword',
          begin: /(using|assembly|command|module|namespace|type)/
        }
      ]
    };

    // Comperison operators & function named parameters.
    const PS_ARGUMENTS = { variants: [
      // PS literals are pretty verbose so it's a good idea to accent them a bit.
      {
        className: 'operator',
        begin: '('.concat(COMPARISON_OPERATORS, ')\\b')
      },
      {
        className: 'literal',
        begin: /(-){1,2}[\w\d-]+/,
        relevance: 0
      }
    ] };

    const HASH_SIGNS = {
      className: 'selector-tag',
      begin: /@\B/,
      relevance: 0
    };

    // It's a very general rule so I'll narrow it a bit with some strict boundaries
    // to avoid any possible false-positive collisions!
    const PS_METHODS = {
      className: 'function',
      begin: /\[.*\]\s*[\w]+[ ]??\(/,
      end: /$/,
      returnBegin: true,
      relevance: 0,
      contains: [
        {
          className: 'keyword',
          begin: '('.concat(
            KEYWORDS.keyword.toString().replace(/\s/g, '|'
            ), ')\\b'),
          endsParent: true,
          relevance: 0
        },
        hljs.inherit(hljs.TITLE_MODE, { endsParent: true })
      ]
    };

    const GENTLEMANS_SET = [
      // STATIC_MEMBER,
      PS_METHODS,
      PS_COMMENT,
      BACKTICK_ESCAPE,
      hljs.NUMBER_MODE,
      QUOTE_STRING,
      APOS_STRING,
      // PS_NEW_OBJECT_TYPE,
      CMDLETS,
      VAR,
      LITERAL,
      HASH_SIGNS
    ];

    const PS_TYPE = {
      begin: /\[/,
      end: /\]/,
      excludeBegin: true,
      excludeEnd: true,
      relevance: 0,
      contains: [].concat(
        'self',
        GENTLEMANS_SET,
        {
          begin: "(" + TYPES.join("|") + ")",
          className: "built_in",
          relevance: 0
        },
        {
          className: 'type',
          begin: /[\.\w\d]+/,
          relevance: 0
        }
      )
    };

    PS_METHODS.contains.unshift(PS_TYPE);

    return {
      name: 'PowerShell',
      aliases: [
        "pwsh",
        "ps",
        "ps1"
      ],
      case_insensitive: true,
      keywords: KEYWORDS,
      contains: GENTLEMANS_SET.concat(
        PS_CLASS,
        PS_FUNCTION,
        PS_USING,
        PS_ARGUMENTS,
        PS_TYPE
      )
    };
  }

  return powershell;

})();

    hljs.registerLanguage('powershell', hljsGrammar);
  })();/*! `sqf` grammar compiled for Highlight.js 11.10.0 */
  (function(){
    var hljsGrammar = (function () {
  'use strict';

  /*
  Language: SQF
  Author: Søren Enevoldsen <senevoldsen90@gmail.com>
  Contributors: Marvin Saignat <contact@zgmrvn.com>, Dedmen Miller <dedmen@dedmen.de>, Leopard20
  Description: Scripting language for the Arma game series
  Website: https://community.bistudio.com/wiki/SQF_syntax
  Category: scripting
  Last update: 07.01.2023, Arma 3 v2.11
  */

  /*
  ////////////////////////////////////////////////////////////////////////////////////////////
    * Author: Leopard20
    
    * Description:
    This script can be used to dump all commands to the clipboard.
    Make sure you're using the Diag EXE to dump all of the commands.
    
    * How to use:
    Simply replace the _KEYWORDS and _LITERAL arrays with the one from this sqf.js file.
    Execute the script from the debug console.
    All commands will be copied to the clipboard.
  ////////////////////////////////////////////////////////////////////////////////////////////
  _KEYWORDS = ['if'];                                                //Array of all KEYWORDS
  _LITERALS = ['west'];                                              //Array of all LITERALS
  _allCommands = createHashMap;
  {
    _type = _x select [0,1];
    if (_type != "t") then {
      _command_lowercase = ((_x select [2]) splitString " ")#(((["n", "u", "b"] find _type) - 1) max 0);
      _command_uppercase = supportInfo ("i:" + _command_lowercase) # 0 # 2;
      _allCommands set [_command_lowercase, _command_uppercase];
    };
  } forEach supportInfo "";
  _allCommands = _allCommands toArray false;
  _allCommands sort true;                                            //sort by lowercase
  _allCommands = ((_allCommands apply {_x#1}) -_KEYWORDS)-_LITERALS; //remove KEYWORDS and LITERALS
  copyToClipboard (str (_allCommands select {_x regexMatch "\w+"}) regexReplace ["""", "'"] regexReplace [",", ",\n"]);
  */

  function sqf(hljs) {
    // In SQF, a local variable starts with _
    const VARIABLE = {
      className: 'variable',
      begin: /\b_+[a-zA-Z]\w*/
    };

    // In SQF, a function should fit myTag_fnc_myFunction pattern
    // https://community.bistudio.com/wiki/Functions_Library_(Arma_3)#Adding_a_Function
    const FUNCTION = {
      className: 'title',
      begin: /[a-zA-Z][a-zA-Z_0-9]*_fnc_[a-zA-Z_0-9]+/
    };

    // In SQF strings, quotes matching the start are escaped by adding a consecutive.
    // Example of single escaped quotes: " "" " and  ' '' '.
    const STRINGS = {
      className: 'string',
      variants: [
        {
          begin: '"',
          end: '"',
          contains: [
            {
              begin: '""',
              relevance: 0
            }
          ]
        },
        {
          begin: '\'',
          end: '\'',
          contains: [
            {
              begin: '\'\'',
              relevance: 0
            }
          ]
        }
      ]
    };

    const TYPES = [
        'call',
        'callExtension',
        'compile',
        'compileFinal',
        'exec',
        'execFSM',
        'execVM',
        'private',
        'spawn',
    ];

    const KEYWORDS = [
      'canSuspend','from','execFSM','call','else','scopeName','uiSleep','breakTo','case','with','sleep','fileExists','assert','waitUntil','scriptName','exec','forEachReversed','to','forEach','throw','execVM','forEachMemberTeam','breakWith','break','terminate','continueWith','do','switch','exitWith','exit','breakOut','forEachMember','isUIContext','scriptDone','catch','spawn','then','while','goto','for','default','step','if','continue','try','halt','loadFile','forEachMemberAgent'
    ];

    const LITERAL = [
      'blufor',
      'civilian',
      'configNull',
      'controlNull',
      'displayNull',
      'diaryRecordNull',
      'east',
      'endl',
      'false',
      'grpNull',
      'independent',
      'lineBreak',
      'locationNull',
      'nil',
      'objNull',
      'opfor',
      'pi',
      'resistance',
      'scriptNull',
      'sideAmbientLife',
      'sideEmpty',
      'sideEnemy',
      'sideFriendly',
      'sideLogic',
      'sideUnknown',
      'taskNull',
      'teamMemberNull',
      'true',
      'west'
    ];

    const BUILT_IN = [
      'setWaypointScript','sendUDPMessage','removeMPEventHandler','addForceGeneratorRTD','ctrlMapAnimDone','getTurretLimits','ctrlSetFontH1','getWaterFillPercentage','boundingBox','importAllGroups','enginesTorqueRTD','onHCGroupSelectionChanged','formation','allPlayers','lnbAddColumn','copyWaypoints','clientOwner','tvSetCurSel','ctrlVisible','selectBestPlaces','markerDrawPriority','lbSetText','magazinesDetail','setAnimSpeedCoef','removeAllOwnedMines','setStatValue','getWPPos','setSimpleTaskDescription','enableCaustics','commandingMenu','hierarchyObjectsCount','setFormDir','camPrepareFocus','uniformContainer','currentWeapon','isSteamMission','deleteResources','screenToWorldDirection','hintC','assignAsCommander','drawLine','animateBay','removePrimaryWeaponItem','countSide','setStarterRTD','numberOfEnginesRTD','displayRemoveEventHandler','curatorWaypointCost','setSide','setMissileTarget','allCameras','countFriendly','allMissionObjects','onEachFrame','hashValue','addMagazine','radioChannelInfo','missionProfileNamespace','onOfficialServer','waypointPosition','ctrlParentControlsGroup','removeAllActions','lightDetachObject','getCameraViewDirection','setTriggerActivation','getEntityInfo','waypointBehaviour','blufor','camPrepareFovRange','unlockAchievement','allowedService','clearItemCargoGlobal','getForcedSpeed','lnbGetColumnsPosition','velocityModelSpace','missionStart','getShotParents','getSubtitleOptions','menuHover','awake','radioChannelSetLabel','ctrlSetFontSecondary','clearVehicleInit','toArray','removeCuratorEditableObjects','agent','setTIParameter','save3DENPreferences','setShadowDistance','getPlayerUID','setMarkerDirLocal','menuSetURL','setGroupIconsSelectable','createVehicleCrew','maxLoad','ctrlSetFontHeightH1','setPilotCameraTarget','lnbSetTextRight','getAllEnv3DSoundControllers','ctrlSetDisabledColor','allUnitsUAV','addAction','getObjectID','forceHitPointsDamageSync','get3DENCamera','setBatteryRTD','lnbSize','safeZoneX','assignedCargo','completedFSM','setWaypointPosition','screenToWorldVector','flagAnimationPhase','pixelGrid','forcedMap','ctrlSetPixelPrecision','setCruiseControl','addBackpackGlobal','isRemoteExecuted','setWaves','ctrlSetTooltip','lnbSetValue','configHierarchy','remove3DENConnection','removeDrawLinks','hideSelection','sideRadio','getPersonUsedDLCs','checkAIFeature','showWaypoint','setMarkerShadow','ctrlBackgroundColor','allowDamage','showRadio','ctrlSetFontHeightSecondary','setPosASLW','ctrlTextWidth','tvTooltip','camSetRelPos','getAmmoCargo','onBriefingGear','closeDialog','getCustomAimCoef','countType','findNearestEnemy','getContainerMaxLoad','lnbSetColumnsPos','localize','nearestMines','removeCuratorEditingArea','getMissionOptions','ATLToASL','kickPlayer','actionKeysEx','createGuardedPoint','commandRadio','endMission','removeWeaponAttachmentCargo','teamSwitch','ctrlSetActiveColor','openDLCPage','backpacks','setVehicleAmmoDef','addSwitchableUnit','camPrepareBank','toString','allControls','lbSetTextRight','waypointName','infoPanels','createMarkerLocal','configFile','ctrlSetFontH5','decayGraphValues','getModelInfo','loadGame','addUserActionEventHandler','assignItem','leaderboardInit','endl','safeZoneWAbs','setHitIndex','disableCollisionWith','skipTime','setDetailMapBlendPars','teamSwitchEnabled','assignedVehicles','findCover','updateDrawIcon','safeZoneY','hcSelected','ctrlSetPositionY','joinSilent','handgunItems','ctrlAutoScrollDelay','setWaypointDescription','params','setUnitAbility','nearSupplies','customChat','count','setCuratorSelected','setCuratorSelectionPreset','getEventHandlerInfo','menuEnabled','getAllSoundControllers','ctrlAt','nearestLocations','waypointAttachedVehicle','dynamicSimulationSystemEnabled','weaponCargo','removeSwitchableUnit','playableSlotsNumber','taskState','moveTime','removeCuratorCameraArea','canAddItemToBackpack','playSound','getWeaponCargo','moveObjectToEnd','clearAllItemsFromBackpack','visibleGPS','addToRemainsCollector','lbSetPictureRightColorSelected','setDebriefingText','disableConversation','ctrlSetFontH3','inPolygon','groupSelectedUnits','setEditorMode','vehicleChat','getObjectType','ropes','remove3DENEventHandler','isMarkedForCollection','get3DENAttribute','getText','say3D','createDiaryLink','toJSON','missileTargetPos','setMarkerDrawPriority','addBackpack','waypointCompletionRadius','setLightFlareMaxDistance','campaignConfigFile','rating','clearBackpackCargo','getNumber','remoteControlled','position','vehicleReceiveRemoteTargets','markerText','ctrlMapAnimAdd','saveGame','setMusicEffect','assignAsTurret','stop','switchableUnits','getEngineTargetRPMRTD','ctrlScrollValues','getPilotCameraTarget','show3DIcons','unitAddons','ctFindRowHeader','menuEnable','setMousePosition','shownMap','getPlayerCloudId','findIf','weightRTD','modelToWorld','reportRemoteTarget','setWaypointFormation','leader','controlNull','attachTo','enableDynamicSimulation','ctrlModelScale','terrainIntersectASL','lnbSetData','isUAVConnected','taskHint','driver','setVisibleIfTreeCollapsed','ctrlSetFontHeightH3','simulationEnabled','doTarget','markerSize','clearForcesRTD','taskType','diaryRecordNull','lnbTextRight','getAssetDLCInfo','allMines','vectorDir','get3DENLinesVisible','objStatus','ctRowControls','enemy','ctrlSetTextSecondary','enableEnvironment','getCruiseControl','addCuratorAddons','toFixed','setVectorDir','get3DENSelected','uniqueUnitItems','addMagazineGlobal','systemTime','getPhysicsCollisionFlag','canStand','doorPhase','aimPos','ammoOnPylon','morale','getLightingAt','ppEffectEnabled','preloadTitleObj','ctrlSetFontHeightH5','gunner','set3DENModelsVisible','removeFromRemainsCollector','ctSetCurSel','magazinesDetailVest','ctrlForegroundColor','setMarkerColor','canMove','assignAsCargo','pixelW','vehicleMoveInfo','lookAt','removeItems','doMove','cos','getWorld','allDeadMen','createSoundSource','getObjectChildren','getBackpackCargo','deleteEditorObject','getLoadedModsInfo','activeTitleEffectParams','waypointShow','allActiveTitleEffects','currentCommand','triggerTimeoutCurrent','actionKeysImages','disableRemoteSensors','allSites','setSimpleTaskDestination','ctrlActivate','showPad','createDisplay','getUnitLoadout','currentMagazine','moonPhase','goggles','execEditorScript','preloadTitleRsc','enableVehicleCargo','scriptNull','nearTargets','setSkill','getCursorObjectParams','merge','vectorModelToWorldVisual','removeAllBinocularItems','tvSetPictureColorSelected','action','unassignItem','vectorUp','textLogFormat','abs','revealMine','exp','hideBehindScripted','hcShowBar','lookAtPos','vectorUpVisual','vectorLinearConversion','setInfoPanel','globalChat','objectCurators','commandSuppressiveFire','lbColor','setPylonsPriority','setAperture','vectorWorldToModelVisual','lightnings','textLog','addEventHandler','weaponsInfo','currentWaypoint','allDead','ctrlMapCursor','newOverlay','lnbSetColorRight','getSoundController','remoteExecutedJIPID','getAllEnvSoundControllers','removeWeaponTurret','disableTIEquipment','ctRowCount','directSay','createDiarySubject','setSpeedMode','setVehicleRadar','targetKnowledge','attachObject','load','setPhysicsCollisionFlag','dynamicSimulationDistanceCoef','ctrlEnable','showCuratorCompass','gearSlotAmmoCount','sendSimpleCommand','east','ctrlTextSecondary','ctrlSetMousePosition','getRespawnVehicleInfo','vest','move','magazinesDetailBackpack','getSlotItemName','switchLight','assignedVehicle','shownWarrant','ambientTemperature','roadAt','openGPS','setCustomWeightRTD','setTrafficSpeed','synchronizedTriggers','getLighting','camUseNVG','saveProfileNamespace','isSimpleObject','sliderSetRange','triggerAmmo','showChat','removeEventHandler','removeItemFromBackpack','waypoints','drawTriangle','setFaceAnimation','createMarker','menuDelete','setCamShakeDefParams','clearItemCargo','setPosition','fadeEnvironment','setCenterOfMass','visibleScoretable','allDiaryRecords','ctrlMapScreenToWorld','switchGesture','missileTarget','ctrlText','onBriefingNotes','allowGetIn','isAutotest','createVehicleLocal','enableTraffic','deleteAt','animate','hcShownBar','selectNoPlayer','isTurnedOut','addMenu','saveStatus','getPilotCameraOpticsMode','simulInClouds','isAutonomous','removeWeaponCargo','unassignTeam','setPosATL','setUnitFreefallHeight','targetsQuery','objectParent','menuExpand','ctrlModel','tvValue','enableDirectionStabilization','viewDistance','setNameSound','getPilotCameraPosition','type','current3DENOperation','ropeAttachEnabled','suppressFor','ctrlEnabled','setObjectViewDistance','hideObject','lbSetPictureColorSelected','systemOfUnits','setFlagSide','linearConversion','ppEffectAdjust','removeBinocularItem','sendCloudRequestServer','jsonToGameValue','showHUD','vectorSide','ropeLength','tvSetSelectColor','calculatePath','getHitIndex','forceAtPositionRTD','flatten','getOxygenRemaining','setMissionOptions','activatedAddons','setVehiclePosition','enginesIsOnRTD','difficultyEnabledRTD','isArray','waypointLoiterRadius','setTimeMultiplier','create3DENEntity','showCompass','addMagazinePool','sentencesEnabled','getTrimOffsetRTD','deleteCenter','ctrlSetFontH4B','getRemoteSensorsDisabled','camSetFov','isEqualTypeAny','onBriefingTeamSwitch','ctrlSetFontHeightH4','primaryWeaponMagazine','setMarkerText','ctrlSetTextColorSecondary','animateDoor','lbSetPicture','tan','waypointLoiterAltitude','setForceGeneratorRTD','removeAllMPEventHandlers','setAirplaneThrottle','lockInventory','cutFadeOut','setVectorDirAndUp','isActionMenuVisible','forceRespawn','flagOwner','setOpticsMode','waypointTimeout','isSensorTargetConfirmed','setCurrentTask','simulWeatherSync','curatorEditableObjects','queryWeaponPool','add3DENConnection','ctrlAutoScrollRewind','leaderboardsRequestUploadScore','getGroupIconParams','ctrlIDC','locked','visibleCompass','ctrlShadow','boundingBoxReal','isAllowedCrewInImmobile','isNotEqualTo','setFormationTask','listRemoteTargets','onMapSingleClick','removeWeapon','currentChannel','benchmark','disableUAVConnectability','lockCargo','angularVelocity','ctSetHeaderTemplate','itemsWithMagazines','isForcedWalk','rainbow','sliderSetPosition','import','lnbValue','getTerrainGrid','ctrlSetFontHeightH6','ctrlSetAutoScrollDelay','showWaypoints','handsHit','namedProperties','tvSortByValueAll','enableRopeAttach','canAddItemToVest','remoteExecCall','atg','titleText','radioChannelSetCallSign','vehicleVarName','ignoreTarget','doFSM','isAutoStartUpEnabledRTD','mapCenterOnCamera','doWatch','addItemCargoGlobal','drawPolygon','addBackpackCargoGlobal','flag','getPosWorld','drawLocation','addGoggles','setPlayable','onBriefingPlan','lockWP','removeAllPrimaryWeaponItems','detach','weapons','parsingNamespace','enableAutoStartUpRTD','setToneMapping','setWPPos','assignedTeam','roleDescription','hcLeader','addMissionEventHandler','customWaypointPosition','simulSetHumidity','setLightnings','taskCompleted','getPlayerID','AISFinishHeal','fuel','ctRemoveRows','drawLaser','and','getCustomSoundController','allowCrewInImmobile','player','getShadowDistance','isInstructorFigureEnabled','openSteamApp','unregisterTask','hcSelectGroup','weaponAccessories','setUnconscious','processInitCommands','flyInHeight','roadsConnectedTo','getAnimSpeedCoef','setMarkerShapeLocal','apertureParams','waypointAttachedObject','currentMagazineDetailTurret','values','setToneMappingParams','min','formLeader','secondaryWeapon','importance','hmd','moveInGunner','setBehaviour','setWaypointLoiterType','ceil','setHit','isNil','triggerStatements','isEqualTo','getPylonMagazines','groups','markerAlpha','removeOwnedMine','setEditorObjectScope','forceWeatherChange','append','displayAddEventHandler','addonFiles','removeHandgunItem','hcSetGroup','lightAttachObject','in','camPrepareDir','nearestLocation','setMarkerAlphaLocal','disableUserInput','getObjectDLC','assignedTarget','setWindStr','cameraInterest','getAllPylonsInfo','selectThrowable','setDammage','setCameraInterest','forceFlagTexture','date','shownWatch','lbDelete','getSuppression','modelToWorldWorld','rankId','missileState','serverCommandExecutable','addMagazineCargoGlobal','resistance','curatorCameraAreaCeiling','vectorDirVisual','setRandomLip','tvSetPictureRightColorDisabled','isIRLaserOn','animationState','addUniform','isEqualRef','setLightConePars','fromJSON','loadCuratorSelectionPreset','get3DENMissionAttributes','getFieldManualStartPage','deleteTarget','showUAVFeed','disableNVGEquipment','getCenterOfMass','playScriptedMission','curatorMouseOver','enableSentences','landResult','sqrt','ctSetData','clear3DENInventory','addCuratorPoints','lineIntersects','createSimpleTask','waypointAttachObject','backpackItems','cutRsc','isWeaponRested','lnbPicture','primaryWeapon','ctrlAutoScrollSpeed','removeBackpackGlobal','createDialog','ctrlShow','soundParams','getCustomSoundControllerCount','animationNames','hcRemoveGroup','createGearDialog','isInRemainsCollector','handgunWeapon','inAreaArray','enablePersonTurret','setLightColor','menuSetPicture','showGPS','configClasses','setDiaryRecordText','onPlayerConnected','createTarget','selectMax','setFace','currentMuzzle','waypointHousePosition','getTIParameters','unitReady','removeWeaponGlobal','addScore','actionName','fogParams','kbAddTopic','createTeam','waypointLoiterType','nearObjectsReady','setUserActionText','setCompassDeclination','modParams','setParticleCircle','finishMissionInit','targetsAggregate','brakesDisabled','addWeaponCargo','ctrlSetURL','allowService','setUnitCombatMode','setSystemOfUnits','lnbSetColor','acos','isAISteeringComponentEnabled','turretUnit','didJIP','ctHeaderCount','timeMultiplier','dialog','commandTarget','pickWeaponPool','swimInDepth','compileScript','pitch','setShotParents','ctrlSetTooltipColorShade','cutText','inputAction','weaponsTurret','listObjects','safeZoneW','activateKey','ln','getDescription','startLoadingScreen','vectorAdd','serverConfigTopLevelEntry','menuText','setTaskResult','nMenuItems','queryItemsPool','closeOverlay','setSimpleTaskAlwaysVisible','groupChat','isNumber','getTerrainInfo','enableSaving','enableSatNormalOnDetail','set','setGroupIdGlobal','fromEditor','setUnitLoadout','forceSpeed','waterDamaged','setHitPointDamage','objectFromNetId','taskName','ctrlAnimateModel','camPreparePos','privateAll','enableSimulation','waypointType','saveVar','getFSMVariable','removeMenuItem','headgear','setUnloadInCombat','setCollisionLight','addMagazineAmmoCargo','playersNumber','setMarkerPos','disableBrakes','lbPictureRight','removeBackpack','lbIsSelected','mineActive','tvClear','setRain','editorSetEventHandler','getMarkerColor','removeAllWeapons','forceWalk','vectorDistanceSqr','worldSize','clearWeaponCargoGlobal','waypointDescription','addCuratorEditableObjects','logNetwork','camCommit','nearObjects','lnbPictureRight','lnbSetPictureColor','secondaryWeaponItems','getSteamFriendsServers','listVehicleSensors','ctRemoveHeaders','addMagazineCargo','configName','language','enableAutoTrimRTD','clearWeaponPool','deleteMarker','infoPanelComponents','allMapMarkers','positionCameraToWorld','ctrlAnimationPhaseModel','set3DENIconsVisible','netId','markerDir','tvCollapse','getMarkerSize','ctHeaderControls','verifySignature','terrainIntersect','commander','cameraOn','deleteWaypoint','engineOn','getUserMFDText','isEqualTypeArray','drawRectangle','setCamShakeParams','fog','setFog','onCommandModeChanged','setPilotCameraRotation','simulCloudOcclusion','backpackCargo','triggerInterval','selectionVectorDirAndUp','weaponsItems','getHitPointDamage','allCurators','openCuratorInterface','setObjectTextureGlobal','cbChecked','delete3DENEntities','vectorDotProduct','getArtilleryComputerSettings','mapAnimDone','slingLoadAssistantShown','onPlayerDisconnected','moveTo','ctrlStyle','nextMenuItemIndex','enableEndDialog','setWaypointCompletionRadius','onPreloadFinished','setTriggerText','ctrlSetTooltipColorBox','getMarkerPos','drawLink','ctrlSetFontH4','camPreloaded','humidity','setPilotCameraDirection','alive','logNetworkTerminate','set3DENAttributes','allowFleeing','isLocalized','setCuratorCameraAreaCeiling','inArea','get3DENConnections','setParticleClass','currentVisionMode','dissolveTeam','triggerTimeout','configSourceMod','setTaskState','knowsAbout','triggerDynamicSimulation','assignTeam','isMissionProfileNamespaceLoaded','setVehicleReceiveRemoteTargets','isNotEqualRef','setDamage','lbSetPictureRightColor','getEnv3DSoundController','commandFire','setCustomSoundController','tvSetText','addWeaponGlobal','airDensityRTD','visibleMap','doFollow','setWaypointCombatMode','rad','groupIconSelectable','format','moveToCompleted','menuURL','didJIPOwner','setArmoryPoints','removeUniform','isFlatEmpty','synchronizeObjectsAdd','assignedGroup','ctrlMapDir','object','isAbleToBreathe','arrayIntersect','setWaypointName','isUsingAISteeringComponent','menuPicture','setWaypointTimeout','menuSetShortcut','worldToModel','magazinesAmmo','className','setVehicleId','preloadCamera','param','locationNull','getRelPos','setDirection','fullCrew','local','triggerText','lockedTurret','allTurrets','lnbSetPictureColorRight','deleteGroup','sendAnalyticEvent','setMarkerPosLocal','getFatigue','ctrlShown','throwables','setBehaviourStrong','setParticleRandom','isAwake','isGroupDeletedWhenEmpty','sunOrMoon','airDensityCurveRTD','moveInDriver','isUAVConnectable','deleteRange','damage','copyFromClipboard','getUnitTrait','setStaminaScheme','unitAimPosition','ctrlSetFontHeight','missionNameSource','ctrlSetText','markAsFinishedOnSteam','getWingsPositionRTD','attackEnabled','setGroupId','removeMissionEventHandler','groupId','groupSelectUnit','setCustomMissionData','addItemPool','expectedDestination','getVideoOptions','formationPosition','requiredVersion','getAllUnitTraits','getPilotCameraRotation','taskDescription','reloadEnabled','addMagazines','get3DENIconsVisible','lnbText','createMissionDisplay','joinString','lockedCameraTo','say','isSaving','libraryCredits','lineIntersectsSurfaces','selectedEditorObjects','isThrowable','canUnloadInCombat','wind','getSensorThreats','setTurretLimits','addWaypoint','ctrlSetFontPB','weaponDirection','getPilotCameraDirection','AGLToASL','waypointTimeoutCurrent','magazinesTurret','estimatedEndServerTime','get3DENMouseOver','ignore3DENHistory','vectorWorldToModel','tvCollapseAll','lbData','lnbAddArray','add3DENLayer','visiblePosition','weaponInertia','menuShortcut','addVehicle','assignAsGunner','playMoveNow','setObjectMaterial','move3DENCamera','createTask','ctrlCreate','reverse','linkItem','allAddonsInfo','secondaryWeaponMagazine','atan','ctrlType','setUserMFDText','civilian','displayUniqueName','safeZoneXAbs','waypointsEnabledUAV','connectToServer','isFilePatchingEnabled','join','sendAUMessage','isEqualType','getSensorTargets','showMap','isHidden','enableSimulationGlobal','getGroupIcon','setRotorBrakeRTD','setMarkerBrushLocal','-','getRelDir','deleteLocation','echo','addBackpackCargo','kbWasSaid','str','currentTasks','lbTextRight','removeAllCuratorCameraAreas','getClientState','set3DENLayer','set3DENGrid','reversedMouseY','magazinesAllTurrets','hintCadet','typeOf','setWindDir','pushBack','nearestObjects','lightIsOn','addWeapon','name','distributionRegion','vectorCrossProduct','setLightIR','clearWeaponCargo','lnbSetText','ctrlSetFontHeightH2','keyImage','clearItemPool','ctrlClassName','hostMission','setAPURTD','setWaypointForceBehaviour','createHashMapObject','menuSetText','setTriggerTimeout','get3DENParent','ctrlFade','sliderPosition','camSetPos','getPosATLVisual','lbSetSelectColorRight','setType','isWalking','getFuelConsumptionCoef','createUnit','setGusts','ctrlTextColor','setDrawIcon','magazineTurretAmmo','getTerrainHeight','setTriggerStatements','deleteTeam','isMultiplayerSolo','ctAddHeader','sideEmpty','backpack','getPlayerScores','firstBackpack','setObjectArguments','addMagazineTurret','radioVolume','modelToWorldVisualWorld','curatorCameraArea','gestureState','publicVariableServer','forceFollowRoad','setFlagOwner','removeSecondaryWeaponItem','getMass','removeAllItems','setTitleEffect','getConnectedUAVUnit','save3DENInventory','createCenter','setActualCollectiveRTD','showCommandingMenu','do3DENAction','onGroupIconOverEnter','accTime','tvSetData','setSlingLoad','getFuelCargo','animatePylon','nearEntities','setHumidity','enableUAVWaypoints','simulCloudDensity','get3DENAttributes','addHandgunItem','ctrlMousePosition','teamType','radioChannelRemove','profileNameSteam','setRank','mapAnimCommit','ctSetRowTemplate','setGroupIconsVisible','markerShadow','setUnitPos','teamMember','collapseObjectTree','moveTarget','setAngularVelocity','ctrlSetTooltipMaxWidth','logEntities','attachedObject','get3DENMissionAttribute','say2D','lockDriver','worldName','environmentEnabled','setWaypointLoiterAltitude','endLoadingScreen','drop','allowDammage','serverTime','canDeployWeapon','ctrlChecked','ctSetValue','switchMove','ctrlSetPositionH','addWeaponPool','registeredTasks','HUDMovementLevels','setVectorUp','getUnloadInCombat','enableMimics','getForcedFlagTexture','shownScoretable','ctrlSetFontH6','setVehicleTIPars','createAgent','isUniformAllowed','radioChannelCreate','playMusic','allAirports','getPosVisual','ctCurSel','lbSort','commandWatch','waypointCombatMode','displayNull','clearOverlay','getBurningValue','taskMarkerOffset','createHashMap','screenshot','lockCameraTo','addPlayerScores','finite','isObjectRTD','backpackMagazines','ropeAttachedTo','camPreload','distance2D','inAreaArrayIndexes','kbAddDatabase','ctrlSetFocus','vectorDiff','getFriend','sideChat','allSimpleObjects','rank','allGroups','setUnitRank','isFormationLeader','captiveNum','inflame','useAudioTimeForMoves','progressSetPosition','leaderboardRequestRowsGlobalAroundUser','selectRandom','ropeSegments','nameSound','isNull','setPilotLight','screenToWorld','fadeSound','enableStressDamage','lnbSortBy','musicVolume','dynamicSimulationDistance','lbSetTooltip','setWeaponReloadingTime','isRemoteExecutedJIP','tvSetSelected','ropeUnwind','setMarkerTypeLocal','restartEditorCamera','addCamShake','regexMatch','cursorTarget','setCamUseTI','createVehicle','isOnRoad','activateAddons','moveInCargo','selectPlayer','tvExpand','get3DENLayer','unitPos','sin','getGraphValues','get3DENLayerEntities','isKeyActive','conversationDisabled','kbReact','currentThrowable','ctrlSetTextSelection','removeAllCuratorEditingAreas','setTargetAge','progressPosition','tvSelection','getTerrainHeightASL','unitIsUAV','unitRecoilCoefficient','selectLeader','setMagazineTurretAmmo','setCurrentWaypoint','actionKeysNamesArray','isGamePaused','lnbSetCurSelRow','worldToModelVisual','setMarkerColorLocal','vehicleReportOwnPosition','clearGroupIcons','isTouchingGround','ctrlMapWorldToScreen','setFeatureType','visiblePositionASL','binocularItems','openDSInterface','addTorque','drawIcon','comment','difficultyOption','flagSide','ctAddRow','getUnitMovesInfo','allCutLayers','get3DENGrid','isStressDamageEnabled','ctrlCommitted','everyBackpack','setObjectScale','remoteExec','setOwner','getLeaning','camSetDir','loadAbs','set3DENLogicType','taskCustomData','isAgent','curatorEditingAreaType','tvCurSel','getCorpseWeaponholders','fadeRadio','setCompassOscillation','confirmSensorTarget','addCuratorEditingArea','numberToDate','isObjectHidden','canTriggerDynamicSimulation','safeZoneH','mineDetectedBy','markerBrush','tvExpandAll','commandMove','clearRadio','camSetDive','setWingForceScaleRTD','removeDiarySubject','preprocessFile','friendly','dateToNumber','respawnVehicle','ctrlMapAnimClear','vectorNormalized','playActionNow','lnbCurSelRow','mod','setVehicleReportRemoteTargets','combatBehaviour','playableUnits','getObjectArgument','setLightBrightness','lbText','lbSetSelectColor','currentMagazineDetail','lineBreak','displayParent','synchronizeTrigger','isClass','onGroupIconClick','getWaterLeakiness','turretOwner','setTriggerArea','setMarkerPolylineLocal','calculatePlayerVisibilityByFriendly','get3DENActionState','face','ctrlSetShadow','toUpper','ASLToAGL','getPiPViewDistance','ctrlSetAngle','onBriefingGroup','cameraView','composeText','set3DENLinesVisible','teamName','atan2','selectWeapon','shownHUD','getCargoIndex','createMenu','setRainbow','markerChannel','findEmptyPositionReady','tvSetValue','setStamina','waypointAttachVehicle','menuSetCheck','missionVersion','unitTurret','setFlagAnimationPhase','ctrlSetForegroundColor','enableTeamSwitch','deleteGroupWhenEmpty','ctrlSetModelDirAndUp','isDamageAllowed','missionDifficulty','ctrlSetFontH6B','lbSize','setTowParent','addCuratorCameraArea','netObjNull','getUnitFreefallInfo','windStr','hasWeapon','debugFSM','saveIdentity','lineIntersectsWith','removeTeamMember','vectorSideVisual','tvSetPictureColor','nearestBuilding','setUserMFDValue','precision','pixelGridNoUIScale','removeAction','setLightDayLight','menuSetAction','enableDebriefingStats','speaker','tvText','hcRemoveAllGroups','lbColorRight','getMissionPath','setWaypointType','enableStamina','briefingName','inflamed','isShowing3DIcons','lnbSetPictureRight','setWaypointSpeed','addItemToUniform','setWaypointLoiterRadius','mapAnimClear','lbSortByValue','menuValue','playerSide','menuClear','customRadio','isCopilotEnabled','ppEffectEnable','toLowerANSI','flagTexture','missionConfigFile','enableDiagLegend','hideBody','savingEnabled','profileNamespace','addWeaponWithAttachmentsCargoGlobal','assignAsDriver','enableAudioFeature','setBrakesRTD','orderGetIn','setSimpleTaskType','magazineCargo','setMarkerPolyline','getWeaponSway','ctrlDelete','ctrlModelDirAndUp','showNewEditorObject','allow3DMode','getLoginStatus','serverCommand','cbSetChecked','getMissionConfigValue','enableChannel','getAttackTarget','wingsForcesRTD','UAVControl','formationLeader','lbClear','initAmbientLife','showWarrant','currentWeaponTurret','getDammage','setWaypointHousePosition','titleObj','addSecondaryWeaponItem','isEqualTypeAll','regexFind','clearBackpackCargoGlobal','updateObjectTree','sideUnknown','lbSetColorRight','tvSort','setCuratorWaypointCost','periscopeElevation','getSpeed','loadMagazine','onDoubleClick','isManualFire','getRepairCargo','unitBackpack','markerShape','controlsGroupCtrl','pi','setAirportSide','rainParams','getStamina','enableAttack','isHideBehindScripted','skillFinal','armoryPoints','missionNamespace','backpackSpaceFor','lbCurSel','ctrlMapPosition','crew','leaderboardsRequestUploadScoreKeepBest','curatorCoef','tvSetPictureRightColor','rain','isVehicleRadarOn','menuSize','focusedCtrl','setTriggerType','setMarkerBrush','setParticleParams','faction','weaponReloadingTime','splitString','lifeState','setParticleFire','setGroupIcon','size','isVehicleCargo','drawArrow','enableGunLights','isTutHintsEnabled','currentTask','ropeAttachedObjects','hideObjectGlobal','isLightOn','removeAllItemsWithMagazines','waypointVisible','isSprintAllowed','allowFileOperations','setMass','isEngineOn','getAssignedCuratorUnit','setWaterFillPercentage','moveInTurret','onPreloadStarted','loadUniform','displayCtrl','removeMagazineTurret','ctrlAddEventHandler','getHit','removeVest','keys','getTextRaw','menuSort','onTeamSwitch','nearestLocationWithDubbing','getElevationOffset','visibleWatch','posScreenToWorld','synchronizedObjects','selectionNames','lnbSetTooltip','eyeDirection','getObjectMaterials','not','freeLook','removeAllContainers','getArray','collect3DENHistory','getArtilleryAmmo','getDLCAssetsUsageByName','laserTarget','lbSetPictureRight','ctrlTextHeight','shownCompass','speed','curatorCamera','tvIsSelected','insideBuilding','batteryChargeRTD','loadConfig','posWorldToScreen','waypointForceBehaviour','setPos','enableInfoPanelComponent','enableEngineArtillery','triggerActivated','getAssignedCuratorLogic','environmentVolume','setSpeaker','ctrlScale','createGroup','getTotalDLCUsageTime','attachedTo','radioEnabled','camSetBank','triggerAttachObject','getVehicleCargo','getPosASLVisual','currentZeroing','displayUpdate','resources','allLODs','globalRadio','saveOverlay','triggerArea','canFire','targets','lineIntersectsObjs','leaderboardRequestRowsFriends','effectiveCommander','hasInterface','curatorRegisteredObjects','agents','shownUAVFeed','shownPad','admin','uniformItems','playAction','gearSlotData','lbSortBy','leaderboardDeInit','emptyPositions','vectorMagnitudeSqr','forceUnicode','setUnitRecoilCoefficient','setFlagTexture','ropeAttachTo','getVehicleTIPars','tvSetPictureRightColorSelected','checkVisibility','getMissionLayers','setVelocity','forgetTarget','setBleedingRemaining','setMarkerTextLocal','asin','currentMagazineTurret','setAutonomous','getGroupIcons','ctrlIDD','ctrlSetPositionW','weaponLowered','showCinemaBorder','leaveVehicle','getSlingLoad','removeAllHandgunItems','menuSetData','sideEnemy','enableAimPrecision','ropeCreate','vehicleRadio','triggerActivation','failMission','throttleRTD','vectorDistance','unitAimPositionVisual','getEnvSoundController','lnbSetPictureColorSelected','canVehicleCargo','camCommitted','forceEnd','deleteMarkerLocal','actionNow','soundVolume','nil','addPublicVariableEventHandler','needService','removeClothing','assignAsCargoIndex','getPos','lnbDeleteColumn','addWeaponCargoGlobal','landAt','matrixTranspose','formationTask','removeAllMusicEventHandlers','independent','setAmmoCargo','sizeOf','moonIntensity','setAttributes','formationDirection','lbSetPictureRightColorDisabled','ctrlSetURLOverlayMode','itemCargo','systemChat','incapacitatedState','lnbAddRow','triggerType','triggerAttachVehicle','setMusicEventHandler','deleteCollection','markerPolyline','configSourceModList','addEditorObject','ppEffectDestroy','setWaypointVisible','getDLCs','detectedMines','setGroupIconParams','set3DENAttribute','set3DENAttachedCursorEntity','publicVariable','getMarkerType','rectangular','setRadioMsg','setCustomAimCoef','difficulty','ctrlSetFont','collectiveRTD','ctrlSetEventHandler','setFromEditor','createHashMapFromArray','addTeamMember','isPlayerSupporter','getAllHitPointsDamage','getWingsOrientationRTD','setMarkerType','titleCut','getDiverState','setEffectiveCommander','stopEngineRTD','shownCuratorCompass','waypointSpeed','worldToScreen','setSoundEffect','setVariable','uiNamespace','waypointFormation','isRealTime','createMine','hintSilent','set3DENSelected','magazinesAmmoCargo','vehicle','ropeEndPosition','insertEditorObject','removeDiaryRecord','setBatteryChargeRTD','openMap','ctData','allDiarySubjects','setCurrentChannel','getOpticsMode','getTurretOpticsMode','soldierMagazines','enableRadio','tvSetColor','getAllOwnedMines','ctrlRemoveAllEventHandlers','isCollisionLightOn','menuCollapse','useAIOperMapObstructionTest','setPosASL','isFlashlightOn','tvAdd','titleRsc','disableDebriefingStats','animationPhase','scudState','ctrlSetFontH3B','synchronizeWaypoint','addHeadgear','drawEllipse','ctClear','fadeSpeech','setHUDMovementLevels','camSetFovRange','owner','moveInCommander','setVehicleAmmo','setVehicleReportOwnPosition','getPosASLW','playMove','backpackContainer','getBoneNames','setEffectCondition','toLower','list','enginesRpmRTD','setPylonLoadout','getPosATL','tvData','vectorModelToWorld','progressLoadingScreen','markerColor','openYoutubeVideo','drawIcon3D','lbSetValue','setMarkerSize','tvSetPicture','getDLCUsageTime','nextWeatherChange','windRTD','camPrepareFov','addItem','serverCommandAvailable','forceWeaponFire','tvPictureRight','setTriggerInterval','sendTaskResult','resetSubgroupDirection','assignedCommander','setSimulWeatherLayers','curatorSelected','camPrepareTarget','triggerAttachedVehicle','getMissionDLCs','enableAI','tvSetTooltip','radioChannelAdd','evalObjectArgument','magazinesAmmoFull','addPrimaryWeaponItem','menuShortcutText','weaponState','add3DENEventHandler','taskAlwaysVisible','is3DENPreview','setDynamicSimulationDistance','camCreate','enableCamShake','ctrlModelVectorSide','loadIdentity','getCompatiblePylonMagazines','nearestObject','doStop','removeGoggles','deActivateKey','vectorCos','parseText','isFinal','magazinesDetailUniform','setPlateNumber','ctrlSetPosition','isStreamFriendlyUIEnabled','ctrlSetTooltipColorText','ctrlURLOverlayMode','curatorPoints','isAimPrecisionEnabled','fogForecast','switchCamera','actionIDs','hint','putWeaponPool','setHorizonParallaxCoef','moveToFailed','inGameUISetEventHandler','setObjectMaterialGlobal','direction','debugLog','setLightAttenuation','getPosWorldVisual','isServer','setTrafficDistance','getUserInfo','gameValueToJson','setDefaultCamera','lnbSetPictureColorSelectedRight','needReload','boundingCenter','getEditorObjectScope','setPosASL2','setWindForce','buildingExit','pixelH','stopSound','time','lnbColor','clearKillConfirmations','setCaptive','doSuppressiveFire','weaponDisassemblyEnabled','dynamicSimulationEnabled','trim','cameraEffectEnableHUD','groupOwner','getMagazineCargo','kbHasTopic','setLocalWindParams','camCommitPrepared','markerPos','getTextureInfo','typeName','setWind','ctrlAngle','disableSerialization','compatibleMagazines','getObjectTextures','setVehicleVarName','taskParent','isSteamOverlayEnabled','getPlayerChannel','doArtilleryFire','apply','selectEditorObject','formatText','userInputDisabled','setCuratorEditingAreaType','setTerrainHeight','underwater','disableMapIndicators','or','shownGPS','everyContainer','inputMouse','ppEffectForceInNVG','isRemoteControlling','setIdentity','lbSetSelected','create3DENComposition','setPlayerRespawnTime','get3DENEntity','cadetMode','processDiaryLink','binocularMagazine','clearMagazineCargoGlobal','lock','intersect','assignedItems','setVelocityTransformation','productVersion','assignToAirport','hasPilotCamera','setVehicleArmor','setLightAmbient','modelToWorldVisual','getDirVisual','setDir','getDLCAssetsUsage','tvSetPictureRight','missionEnd','focusOn','menuData','attachedObjects','lbSetPictureColor','ropeDestroy','members','setEngineRpmRTD','set3DENMissionAttributes','bezierInterpolation','addForce','setPitch','playSound3D','configSourceAddonList','image','sendCloudRequestClient','setDriveOnPath','reload','captive','forceAddUniform','getObjectViewDistance','exportLandscapeXYZ','getStatValue','leaderboardRequestRowsGlobal','getRotorBrakeRTD','displayChild','currentPilot','ctrlMapSetPosition','setMarkerDir','sendTask','vestContainer','buttonAction','setPilotCameraOpticsMode','ctrlSetScale','ctrlSetAutoScrollRewind','true','setMarkerShape','resize','setCombatMode','exportJIPMessages','setWeaponZeroing','setFuelConsumptionCoef','sliderRange','deleteVehicle','removeItem','land','airportSide','animateSource','speechVolume','ctFindHeaderRows','configNull','removeMagazineGlobal','lockIdentity','matrixMultiply','isAutoTrimOnRTD','waves','diarySubjectExists','setUnitTrait','removeAllAssignedItems','getRoadInfo','setObjectTexture','sideAmbientLife','configProperties','setRectangular','disableAI','ASLToATL','unassignVehicle','setMaxLoad','setGroupOwner','setVelocityModelSpace','ropesAttachedTo','parseSimpleArray','setMarkerAlpha','sendCloudRequest','unassignCurator','group','camTarget','aimedAtTarget','cursorObject','getObjectProxy','curatorEditingArea','playerTargetLock','moveInAny','rotorsRpmRTD','addWeaponTurret','camSetTarget','canAdd','showLegend','max','pixelGridBase','waypointStatements','connectTerminalToUAV','registerTask','setPosWorld','vehicleCargoEnabled','getCurrentPlayerLevel','setRepairCargo','getHideFrom','setFSMVariable','ctrlHTMLLoaded','updateMenuItem','is3DEN','ppEffectCommitted','setImportance','ammo','text','addItemCargo','setTrafficDensity','items','removeAllCuratorAddons','simpleTasks','setLightIntensity','camConstuctionSetParams','missionName','getObjectFOV','get3DENEntityID','ctrlURL','terrainIntersectAtASL','overcast','reveal','getPlayerLevel','addMusicEventHandler','uniformMagazines','kbRemoveTopic','resetCamShake','loadBackpack','waypointScript','getPlayerVoNVolume','setTurretOpticsMode','weaponAccessoriesCargo','setPiPViewDistance','nearRoads','ropeDetach','ctrlMapScale','serverNamespace','clearMagazinePool','getObjectScale','west','getConnectedUAV','removeSimpleTask','lbTooltip','curatorSelectionPreset','taskChildren','isEqualTypeParams','setSpeech','ctrlSetFade','kbAddDatabaseTargets','isStaminaEnabled','setWaterLeakiness','estimatedTimeLeft','menuAction','side','getMusicPlayedTime','setThrottleRTD','tvDelete','isPlayer','mapGridPosition','freeExtension','squadParams','setVehicleCargo','setAmmo','setLeader','commandStop','opfor','libraryDisclaimers','behaviour','removeCuratorAddons','ctrlSetFontH1B','vectorMagnitude','ctrlSetScrollValues','selectMin','sideLogic','setAngularVelocityModelSpace','setFuelCargo','allDisplays','actionKeysNames','isVehicleSensorEnabled','camSetFocus','lockedInventory','removeHeadgear','lnbSort','floor','allVariables','assignCurator','getMissionConfig','removeMusicEventHandler','findEmptyPosition','setDestination','sliderSetSpeed','enableIRLasers','skill','deleteIdentity','removeMagazines','toUpperANSI','getEditorCamera','addScoreSide','displaySetEventHandler','curatorAddons','tvSetPictureColorDisabled','lbSetData','getAudioOptionVolumes','setTaskMarkerOffset','moveOut','remoteControl','eyePos','setOvercast','flyInHeightASL','ctrlMapAnimCommit','angularVelocityModelSpace','lnbClear','fireAtTarget','ropeSetCargoMass','setVehicleInit','handgunMagazine','kbTell','createLocation','lnbColorRight','removeAllUserActionEventHandlers','getCalculatePlayerVisibilityByFriendly','copyToClipboard','isBleeding','parseNumber','primaryWeaponItems','deg','ctrlSetFontH2B','setOxygenRemaining','leaderboardGetRows','lockedCargo','setHideBehind','commandFSM','round','saveMissionProfileNamespace','selectRandomWeighted','getDir','setVehicleLock','setPlayerVoNVolume','showWatch','formationMembers','addVest','doGetOut','taskDestination','hasCustomFace','lbAdd','removeAllMissionEventHandlers','score','setMarkerShadowLocal','equipmentDisabled','false','enableCollisionWith','getMissionLayerEntities','loadOverlay','setFriend','createTrigger','unitsBelowHeight','setDynamicSimulationDistanceCoef','deleteVehicleCrew','surfaceNormal','removeGroupIcon','fire','playGesture','insert','profileName','getUserMFDValue','collisionDisabledWith','setSuppression','onGroupIconOverLeave','scoreSide','preloadObject','ctrlTextSelection','setWaypointStatements','systemTimeUTC','gearIDCAmmoCount','setCombatBehaviour','getShotInfo','ctrlParent','setSimpleTaskCustomData','commandChat','countEnemy','unitCombatMode','glanceAt','ppEffectCreate','setViewDistance','getOrDefault','distanceSqr','mapAnimAdd','addItemToBackpack','shownChat','menuAdd','windDir','taskNull','isWeaponDeployed','getSelectionBones','setName','clearMagazineCargo','switchAction','publicVariableClient','vestItems','ctrlSetFontP','compatibleItems','WFSideText','getPlayerUIDOld','assignedVehicleRole','getPosASL','supportInfo','ctrlSetModelScale','playerRespawnTime','setDiarySubjectPicture','getClientStateNumber','hcAllGroups','hcGroupParams','menuSetValue','setLightVolumeShape','lbSelection','ropeCut','setObjectProxy','findAny','tg','getTextWidth','entities','channelEnabled','debriefingText','getAimingCoef','addWeaponItem','enableWeaponDisassembly','set3DENMissionAttribute','selectDiarySubject','assignedDriver','isLaserOn','ctrlSetChecked','createSimpleObject','locationPosition','isDLCAvailable','distance','allEnv3DSoundSources','get','setText','infoPanelComponentEnabled','serverName','cheatsEnabled','getSoundControllerResult','rotorsForcesRTD','getVariable','cutObj','tvPicture','overcastForecast','animationSourcePhase','sliderSpeed','fleeing','pose','camDestroy','ctrlMapMouseOver','commitOverlay','grpNull','findEditorObject','forceCadetDifficulty','doFire','queryMagazinePool','countUnknown','shownArtilleryComputer','setMimic','preprocessFileLineNumbers','isBurning','difficultyEnabled','speedMode','regexReplace','lbSetPictureColorDisabled','lnbData','addBinocularItem','addRating','limitSpeed','getPlateNumber','runInitScript','enableVehicleSensor','addCuratorSelected','someAmmo','weaponsItemsCargo','isAutoHoverOn','lbSetCurSel','set3DENObjectType','uniform','synchronizedWaypoints','pushBackUnique','addOwnedMine','commandFollow','setSimpleTaskTarget','ctrlSetFontH5B','createMPCampaignDisplay','commandGetOut','currentWeaponMode','ctrlSetModel','removeMagazine','magazines','setUnitPosWeak','vectorFromTo','stopped','setFormation','isKindOf','enableFatigue','tvSortAll','enableDynamicSimulationSystem','setApertureNew','ctrlSetPositionX','all3DENEntities','removeItemFromVest','getCorpse','addWeaponWithAttachmentsCargo','surfaceIsWater','showScoretable','localNamespace','log','velocity','removeItemFromUniform','joinAsSilent','removeAllEventHandlers','forceMap','findDisplay','tvSortByValue','vehicles','setWaypointBehaviour','loadVest','ropeUnwound','binocular','setTerrainGrid','leaderboardState','keyName','joinAs','removeAll3DENEventHandlers','addResources','getMousePosition','camCommand','deleteSite','setFuel','setConvoySeparation','isAppSubscribed','getArtilleryETA','setDropInterval','menuChecked','enginesPowerRTD','sideFriendly','isPiPEnabled','loadStatus','getNextId','setLightFlareSize','drawLine3D','edit3DENMissionAttributes','teamMemberNull','getBleedingRemaining','ctrlSetBackgroundColor','combatMode','addMPEventHandler','getEditorMode','buttonSetAction','load3DENScenario','enableCopilot','playSoundUI','lbPicture','createSite','enableReload','canAddItemToUniform','inputController','lockTurret','infoPanel','setCameraEffect','setMissileTargetPos','setWantedRPMRTD','vehicleReportRemoteTargets','taskResult','currentNamespace','lnbSortByValue','camPrepareDive','isDedicated','remoteExecutedOwner','ctValue','buildingPos','fadeMusic','showSubtitles','playMission','getDebriefingText','particlesQuality','select','enableAIFeature','closeDisplay','assignedGunner','useAISteeringComponent','allUsers','lnbSetPicture','getResolution','allUnits','actionParams','lnbDeleteRow','deleteStatus','removeAllSecondaryWeaponItems','ctrlSetFontH2','onShowNewObject','directionStabilizationEnabled','forceGeneratorRTD','getItemCargo','find','addItemToVest','is3DENMultiplayer','setAccTime','editObject','ctrlSetTextColor','setAmmoOnPylon','vectorMultiply','groupRadio','selectWeaponTurret','ctrlRemoveEventHandler','objNull','sort','ctrlCommit','groupIconsVisible','titleFadeOut','setDate','displayRemoveAllEventHandlers','priority','camPrepareRelPos','unlinkItem','htmlLoad','setPiPEffect','gusts','inRangeOfArtillery','ctrlFontHeight','setMarkerSizeLocal','shownRadio','canSlingLoad','saveJoysticks','allObjects','preloadSound','shownSubtitles','stance','ctrlTooltip','fillWeaponsFromPool','isText','enableUAVConnectability','ctrlSetAutoScrollSpeed','tvCount','turretLocal','groupFromNetId','addGroupIcon','commandArtilleryFire','getOrDefaultCall','random','vestMagazines','nearestTerrainObjects','remove3DENLayer','configOf','cameraEffect','lockedDriver','setSize','elevatePeriscope','dayTime','allowSprint','selectionPosition','removeMagazinesTurret','clear3DENAttribute','lbValue','units','setFatigue','isGameFocused','ctrlSetStructuredText','createDiaryRecord','surfaceTexture','actionKeys','lbSetColor','setLightUseFlare','removeDrawIcon','addLiveStats','setTrafficGap','inheritsFrom','synchronizeObjectsRemove','ctrlPosition','teams','addMenuItem','serverStartMission','cancelSimpleTaskDestination','removeUserActionEventHandler','getAnimAimPrecision','airplaneThrottle','ppEffectCommit','surfaceType','setCuratorCoef','allowCuratorLogicIgnoreAreas','getTowParent','markerType','isMultiplayer'
    ];
    
    // list of keywords from:
    // https://community.bistudio.com/wiki/PreProcessor_Commands
    const PREPROCESSOR = {
      className: 'meta',
      begin: /#\s*[a-z]+\b/,
      end: /$/,
      keywords: 'define undef ifdef ifndef else endif include if',
      contains: [
        {
          begin: /\\\n/,
          relevance: 0
        },
        hljs.inherit(STRINGS, { className: 'string' }),
        {
          begin: /<[^\n>]*>/,
          end: /$/,
          illegal: '\\n'
        },
        hljs.C_LINE_COMMENT_MODE,
        hljs.C_BLOCK_COMMENT_MODE
      ]
    };
    
    return {
      name: 'SQF',
      case_insensitive: true,
      keywords: {
        type: TYPES,
        keyword: KEYWORDS,
        built_in: BUILT_IN,
        literal: LITERAL
      },
      contains: [
        hljs.C_LINE_COMMENT_MODE,
        hljs.C_BLOCK_COMMENT_MODE,
        hljs.NUMBER_MODE,
        VARIABLE,
        FUNCTION,
        STRINGS,
        PREPROCESSOR
      ],
      illegal: [
        //$ is only valid when used with Hex numbers (e.g. $FF)
        /\$[^a-fA-F0-9]/, 
        /\w\$/,
        /\?/,      //There's no ? in SQF
        /@/,       //There's no @ in SQF
        // Brute-force-fixing the build error. See https://github.com/highlightjs/highlight.js/pull/3193#issuecomment-843088729
        / \| /,
        // . is only used in numbers
        /[a-zA-Z_]\./,
        /\:\=/,
        /\[\:/
      ]
    };
  }

  return sqf;

})();

    hljs.registerLanguage('sqf', hljsGrammar);
  })();/*! `yaml` grammar compiled for Highlight.js 11.10.0 */
  (function(){
    var hljsGrammar = (function () {
  'use strict';

  /*
  Language: YAML
  Description: Yet Another Markdown Language
  Author: Stefan Wienert <stwienert@gmail.com>
  Contributors: Carl Baxter <carl@cbax.tech>
  Requires: ruby.js
  Website: https://yaml.org
  Category: common, config
  */
  function yaml(hljs) {
    const LITERALS = 'true false yes no null';

    // YAML spec allows non-reserved URI characters in tags.
    const URI_CHARACTERS = '[\\w#;/?:@&=+$,.~*\'()[\\]]+';

    // Define keys as starting with a word character
    // ...containing word chars, spaces, colons, forward-slashes, hyphens and periods
    // ...and ending with a colon followed immediately by a space, tab or newline.
    // The YAML spec allows for much more than this, but this covers most use-cases.
    const KEY = {
      className: 'attr',
      variants: [
        // added brackets support 
        { begin: /\w[\w :()\./-]*:(?=[ \t]|$)/ },
        { // double quoted keys - with brackets
          begin: /"\w[\w :()\./-]*":(?=[ \t]|$)/ },
        { // single quoted keys - with brackets
          begin: /'\w[\w :()\./-]*':(?=[ \t]|$)/ },
      ]
    };

    const TEMPLATE_VARIABLES = {
      className: 'template-variable',
      variants: [
        { // jinja templates Ansible
          begin: /\{\{/,
          end: /\}\}/
        },
        { // Ruby i18n
          begin: /%\{/,
          end: /\}/
        }
      ]
    };
    const STRING = {
      className: 'string',
      relevance: 0,
      variants: [
        {
          begin: /'/,
          end: /'/
        },
        {
          begin: /"/,
          end: /"/
        },
        { begin: /\S+/ }
      ],
      contains: [
        hljs.BACKSLASH_ESCAPE,
        TEMPLATE_VARIABLES
      ]
    };

    // Strings inside of value containers (objects) can't contain braces,
    // brackets, or commas
    const CONTAINER_STRING = hljs.inherit(STRING, { variants: [
      {
        begin: /'/,
        end: /'/
      },
      {
        begin: /"/,
        end: /"/
      },
      { begin: /[^\s,{}[\]]+/ }
    ] });

    const DATE_RE = '[0-9]{4}(-[0-9][0-9]){0,2}';
    const TIME_RE = '([Tt \\t][0-9][0-9]?(:[0-9][0-9]){2})?';
    const FRACTION_RE = '(\\.[0-9]*)?';
    const ZONE_RE = '([ \\t])*(Z|[-+][0-9][0-9]?(:[0-9][0-9])?)?';
    const TIMESTAMP = {
      className: 'number',
      begin: '\\b' + DATE_RE + TIME_RE + FRACTION_RE + ZONE_RE + '\\b'
    };

    const VALUE_CONTAINER = {
      end: ',',
      endsWithParent: true,
      excludeEnd: true,
      keywords: LITERALS,
      relevance: 0
    };
    const OBJECT = {
      begin: /\{/,
      end: /\}/,
      contains: [ VALUE_CONTAINER ],
      illegal: '\\n',
      relevance: 0
    };
    const ARRAY = {
      begin: '\\[',
      end: '\\]',
      contains: [ VALUE_CONTAINER ],
      illegal: '\\n',
      relevance: 0
    };

    const MODES = [
      KEY,
      {
        className: 'meta',
        begin: '^---\\s*$',
        relevance: 10
      },
      { // multi line string
        // Blocks start with a | or > followed by a newline
        //
        // Indentation of subsequent lines must be the same to
        // be considered part of the block
        className: 'string',
        begin: '[\\|>]([1-9]?[+-])?[ ]*\\n( +)[^ ][^\\n]*\\n(\\2[^\\n]+\\n?)*'
      },
      { // Ruby/Rails erb
        begin: '<%[%=-]?',
        end: '[%-]?%>',
        subLanguage: 'ruby',
        excludeBegin: true,
        excludeEnd: true,
        relevance: 0
      },
      { // named tags
        className: 'type',
        begin: '!\\w+!' + URI_CHARACTERS
      },
      // https://yaml.org/spec/1.2/spec.html#id2784064
      { // verbatim tags
        className: 'type',
        begin: '!<' + URI_CHARACTERS + ">"
      },
      { // primary tags
        className: 'type',
        begin: '!' + URI_CHARACTERS
      },
      { // secondary tags
        className: 'type',
        begin: '!!' + URI_CHARACTERS
      },
      { // fragment id &ref
        className: 'meta',
        begin: '&' + hljs.UNDERSCORE_IDENT_RE + '$'
      },
      { // fragment reference *ref
        className: 'meta',
        begin: '\\*' + hljs.UNDERSCORE_IDENT_RE + '$'
      },
      { // array listing
        className: 'bullet',
        // TODO: remove |$ hack when we have proper look-ahead support
        begin: '-(?=[ ]|$)',
        relevance: 0
      },
      hljs.HASH_COMMENT_MODE,
      {
        beginKeywords: LITERALS,
        keywords: { literal: LITERALS }
      },
      TIMESTAMP,
      // numbers are any valid C-style number that
      // sit isolated from other words
      {
        className: 'number',
        begin: hljs.C_NUMBER_RE + '\\b',
        relevance: 0
      },
      OBJECT,
      ARRAY,
      STRING
    ];

    const VALUE_MODES = [ ...MODES ];
    VALUE_MODES.pop();
    VALUE_MODES.push(CONTAINER_STRING);
    VALUE_CONTAINER.contains = VALUE_MODES;

    return {
      name: 'YAML',
      case_insensitive: true,
      aliases: [ 'yml' ],
      contains: MODES
    };
  }

  return yaml;

})();

    hljs.registerLanguage('yaml', hljsGrammar);
  })();
