(globalThis.TURBOPACK||(globalThis.TURBOPACK=[])).push(["object"==typeof document?document.currentScript:void 0,24420,e=>{"use strict";e.i(96288),e.i(61838);var t=e.i(48232);e.i(88361);let r=String.raw,i=r`\[\^?`,a=`c.? | C(?:-.?)?|${r`[pP]\{(?:\^?[-\x20_]*[A-Za-z][-\x20\w]*\})?`}|${r`x[89A-Fa-f]\p{AHex}(?:\\x[89A-Fa-f]\p{AHex})*`}|${r`u(?:\p{AHex}{4})? | x\{[^\}]*\}? | x\p{AHex}{0,2}`}|${r`o\{[^\}]*\}?`}|${r`\d{1,3}`}`;RegExp(r`
  \\ (?:
    ${a}
    | [gk]<[^>]*>?
    | [gk]'[^']*'?
    | .
  )
  | \( (?:
    \? (?:
      [:=!>({]
      | <[=!]
      | <[^>]*>
      | '[^']*'
      | ~\|?
      | #(?:[^)\\]|\\.?)*
      | [^:)]*[:)]
    )?
    | \*[^\)]*\)?
  )?
  | (?:${/[?*+][?+]?|\{(?:\d+(?:,\d*)?|,\d+)\}\??/.source})+
  | ${i}
  | .
`.replace(/\s+/g,""),"gsu"),RegExp(r`
  \\ (?:
    ${a}
    | .
  )
  | \[:(?:\^?\p{Alpha}+|\^):\]
  | ${i}
  | &&
  | .
`.replace(/\s+/g,""),"gsu");let n=String.raw`\(\?(?:[:=!>A-Za-z\-]|<[=!]|\(DEFINE\))`;Object.freeze({DEFAULT:"DEFAULT",CHAR_CLASS:"CHAR_CLASS"}),RegExp(String.raw`(?<noncapturingStart>${n})|(?<capturingStart>\((?:\?<[^>]+>)?)|\\?.`,"gsu");let s=String.raw`(?:[?*+]|\{\d+(?:,\d*)?\})`;RegExp(String.raw`
\\(?: \d+
  | c[A-Za-z]
  | [gk]<[^>]+>
  | [pPu]\{[^\}]+\}
  | u[A-Fa-f\d]{4}
  | x[A-Fa-f\d]{2}
  )
| \((?: \? (?: [:=!>]
  | <(?:[=!]|[^>]+>)
  | [A-Za-z\-]+:
  | \(DEFINE\)
  ))?
| (?<qBase>${s})(?<qMod>[?+]?)(?<invalidQ>[?*+\{]?)
| \\?.
`.replace(/\s+/g,""),"gsu");let o=String.raw,l=o`\\g<(?<gRNameOrNum>[^>&]+)&R=(?<gRDepth>[^>]+)>`,c=o`\(\?R=(?<rDepth>[^\)]+)\)|${l}`,p=o`\(\?<(?![=!])(?<captureName>[^>]+)>`;o`${p}|(?<unnamed>\()(?!\?)`,RegExp(o`${p}|${c}|\(\?|\\?.`,"gsu");var h=String.fromCodePoint,g=String.raw,d={},u=globalThis.RegExp;function m(e){let t=h(e);return[t.toLowerCase(),t]}function C(e,t){return(function(e,t){let r=[];for(let i=e;i<=t;i++)r.push(i);return r})(e,t).map(e=>m(e))}d.flagGroups=(()=>{try{new u("(?i:)")}catch{return!1}return!0})(),d.unicodeSets=(()=>{try{new u("[[]]","v")}catch{return!1}return!0})(),d.bugFlagVLiteralHyphenIsRange=!!d.unicodeSets&&(()=>{try{new u(g`[\d\-a]`,"v")}catch{return!0}return!1})(),d.bugNestedClassIgnoresNegation=d.unicodeSets&&new u("[[^a]]","v").test("a"),h(304),h(305),g`[\p{L}\p{M}\p{N}\p{Pc}]`,`C Other
Cc Control cntrl
Cf Format
Cn Unassigned
Co Private_Use
Cs Surrogate
L Letter
LC Cased_Letter
Ll Lowercase_Letter
Lm Modifier_Letter
Lo Other_Letter
Lt Titlecase_Letter
Lu Uppercase_Letter
M Mark Combining_Mark
Mc Spacing_Mark
Me Enclosing_Mark
Mn Nonspacing_Mark
N Number
Nd Decimal_Number digit
Nl Letter_Number
No Other_Number
P Punctuation punct
Pc Connector_Punctuation
Pd Dash_Punctuation
Pe Close_Punctuation
Pf Final_Punctuation
Pi Initial_Punctuation
Po Other_Punctuation
Ps Open_Punctuation
S Symbol
Sc Currency_Symbol
Sk Modifier_Symbol
Sm Math_Symbol
So Other_Symbol
Z Separator
Zl Line_Separator
Zp Paragraph_Separator
Zs Space_Separator
ASCII
ASCII_Hex_Digit AHex
Alphabetic Alpha
Any
Assigned
Bidi_Control Bidi_C
Bidi_Mirrored Bidi_M
Case_Ignorable CI
Cased
Changes_When_Casefolded CWCF
Changes_When_Casemapped CWCM
Changes_When_Lowercased CWL
Changes_When_NFKC_Casefolded CWKCF
Changes_When_Titlecased CWT
Changes_When_Uppercased CWU
Dash
Default_Ignorable_Code_Point DI
Deprecated Dep
Diacritic Dia
Emoji
Emoji_Component EComp
Emoji_Modifier EMod
Emoji_Modifier_Base EBase
Emoji_Presentation EPres
Extended_Pictographic ExtPict
Extender Ext
Grapheme_Base Gr_Base
Grapheme_Extend Gr_Ext
Hex_Digit Hex
IDS_Binary_Operator IDSB
IDS_Trinary_Operator IDST
ID_Continue IDC
ID_Start IDS
Ideographic Ideo
Join_Control Join_C
Logical_Order_Exception LOE
Lowercase Lower
Math
Noncharacter_Code_Point NChar
Pattern_Syntax Pat_Syn
Pattern_White_Space Pat_WS
Quotation_Mark QMark
Radical
Regional_Indicator RI
Sentence_Terminal STerm
Soft_Dotted SD
Terminal_Punctuation Term
Unified_Ideograph UIdeo
Uppercase Upper
Variation_Selector VS
White_Space space
XID_Continue XIDC
XID_Start XIDS`.split(/\s/).map(e=>[e.replace(/[- _]+/g,"").toLowerCase(),e]),h(383),h(383),h(223),h(7838),h(107),h(8490),h(229),h(8491),h(969),h(8486),[m(453),m(456),m(459),m(498),...C(8072,8079),...C(8088,8095),...C(8104,8111),m(8124),m(8140),m(8188)],g`[\p{Alpha}\p{Nd}]`,g`\p{Alpha}`,g`\p{ASCII}`,g`[\p{Zs}\t]`,g`\p{Cc}`,g`\p{Nd}`,g`[\P{space}&&\P{Cc}&&\P{Cn}&&\P{Cs}]`,g`\p{Lower}`,g`[[\P{space}&&\P{Cc}&&\P{Cn}&&\P{Cs}]\p{Zs}]`,g`[\p{P}\p{S}]`,g`\p{space}`,g`\p{Upper}`,g`[\p{Alpha}\p{M}\p{Nd}\p{Pc}]`,g`\p{AHex}`,g`\t`,g`\n`,g`\v`,g`\f`,g`\r`,g`\u2028`,g`\u2029`,g`\uFEFF`,(class e extends RegExp{#e=new Map;#t=null;#r;#i=null;#a=null;rawOptions={};get source(){return this.#r||"(?:)"}constructor(t,r,i){const a=!!i?.lazyCompile;if(t instanceof RegExp){if(i)throw Error("Cannot provide options when copying a regexp");super(t,r),this.#r=t.source,t instanceof e&&(this.#e=t.#e,this.#i=t.#i,this.#a=t.#a,this.rawOptions=t.rawOptions)}else{const e={hiddenCaptures:[],strategy:null,transfers:[],...i};super(a?"":t,r),this.#r=t,this.#e=function(e,t){let r=new Map;for(let t of e)r.set(t,{hidden:!0});for(let[e,a]of t)for(let t of a){var i;(i={},!r.has(t)&&r.set(t,i),r.get(t)).transferTo=e}return r}(e.hiddenCaptures,e.transfers),this.#a=e.strategy,this.rawOptions=i??{}}a||(this.#t=this)}exec(t){if(!this.#t){let{lazyCompile:t,...r}=this.rawOptions;this.#t=new e(this.#r,this.flags,r)}let r=this.global||this.sticky,i=this.lastIndex;if("clip_search"===this.#a&&r&&i){this.lastIndex=0;let e=this.#n(t.slice(i));return e&&(function(e,t,r,i){if(e.index+=t,e.input=r,i){let r=e.indices;for(let e=0;e<r.length;e++){let i=r[e];i&&(r[e]=[i[0]+t,i[1]+t])}let i=r.groups;i&&Object.keys(i).forEach(e=>{let r=i[e];r&&(i[e]=[r[0]+t,r[1]+t])})}}(e,i,t,this.hasIndices),this.lastIndex+=i),e}return this.#n(t)}#n(e){let t;this.#t.lastIndex=this.lastIndex;let r=super.exec.call(this.#t,e);if(this.lastIndex=this.#t.lastIndex,!r||!this.#e.size)return r;let i=[...r];r.length=1,this.hasIndices&&(t=[...r.indices],r.indices.length=1);let a=[0];for(let e=1;e<i.length;e++){let{hidden:n,transferTo:s}=this.#e.get(e)??{};if(n?a.push(null):(a.push(r.length),r.push(i[e]),this.hasIndices&&r.indices.push(t[e])),s&&void 0!==i[e]){let n=a[s];if(!n)throw Error(`Invalid capture transfer to "${n}"`);if(r[n]=i[e],this.hasIndices&&(r.indices[n]=t[e]),r.groups){this.#i||(this.#i=function(e){let t,r=/(?<capture>\((?:\?<(?![=!])(?<name>[^>]+)>|(?!\?)))|\\?./gsu,i=new Map,a=0,n=0;for(;t=r.exec(e);){let{0:e,groups:{capture:r,name:s}}=t;"["===e?a++:a?"]"===e&&a--:r&&(n++,s&&i.set(n,s))}return i}(this.source));let a=this.#i.get(s);a&&(r.groups[a]=i[e],this.hasIndices&&(r.indices.groups[a]=t[e]))}}}return r}}),e.s([],18433),e.i(18433);var _=e.i(77313),S=e.i(45546),T=e.i(76114);e.s(["ShikiError",()=>S.ShikiError,"addClassToHast",()=>_.addClassToHast,"applyColorReplacements",()=>T.applyColorReplacements,"codeToHast",()=>_.codeToHast,"codeToHtml",()=>_.codeToHtml,"codeToTokens",()=>_.codeToTokens,"codeToTokensBase",()=>_.codeToTokensBase,"codeToTokensWithThemes",()=>T.codeToTokensWithThemes,"createBundledHighlighter",()=>_.createBundledHighlighter,"createCssVariablesTheme",()=>_.createCssVariablesTheme,"createHighlighterCore",()=>_.createHighlighterCore,"createHighlighterCoreSync",()=>_.createHighlighterCoreSync,"createPositionConverter",()=>_.createPositionConverter,"createShikiInternal",()=>T.createShikiInternal,"createShikiInternalSync",()=>T.createShikiInternalSync,"createShikiPrimitive",()=>T.createShikiPrimitive,"createShikiPrimitiveAsync",()=>T.createShikiPrimitiveAsync,"createSingletonShorthands",()=>_.createSingletonShorthands,"flatTokenVariants",()=>_.flatTokenVariants,"getLastGrammarState",()=>T.getLastGrammarState,"getSingletonHighlighterCore",()=>_.getSingletonHighlighterCore,"getTokenStyleObject",()=>_.getTokenStyleObject,"guessEmbeddedLanguages",()=>_.guessEmbeddedLanguages,"hastToHtml",()=>_.hastToHtml,"isNoneTheme",()=>T.isNoneTheme,"isPlainLang",()=>T.isPlainLang,"isSpecialLang",()=>T.isSpecialLang,"isSpecialTheme",()=>T.isSpecialTheme,"makeSingletonHighlighter",()=>_.makeSingletonHighlighter,"makeSingletonHighlighterCore",()=>_.makeSingletonHighlighterCore,"normalizeGetter",()=>T.normalizeGetter,"normalizeTheme",()=>T.normalizeTheme,"resolveColorReplacements",()=>T.resolveColorReplacements,"splitLines",()=>T.splitLines,"splitToken",()=>_.splitToken,"splitTokens",()=>_.splitTokens,"stringifyTokenStyle",()=>_.stringifyTokenStyle,"toArray",()=>T.toArray,"tokenizeAnsiWithTheme",()=>_.tokenizeAnsiWithTheme,"tokenizeWithTheme",()=>T.tokenizeWithTheme,"tokensToHast",()=>_.tokensToHast,"transformerDecorations",()=>_.transformerDecorations],90423),e.i(90423),e.s(["createHighlighter",()=>t.createHighlighter],24420)}]);