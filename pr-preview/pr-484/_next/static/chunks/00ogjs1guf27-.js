(globalThis.TURBOPACK||(globalThis.TURBOPACK=[])).push(["object"==typeof document?document.currentScript:void 0,24420,e=>{"use strict";e.i(96288),e.i(61838);var t=e.i(48232);e.i(88361);let r=String.raw,a=r`\[\^?`,i=`c.? | C(?:-.?)?|${r`[pP]\{(?:\^?[-\x20_]*[A-Za-z][-\x20\w]*\})?`}|${r`x[89A-Fa-f]\p{AHex}(?:\\x[89A-Fa-f]\p{AHex})*`}|${r`u(?:\p{AHex}{4})? | x\{[^\}]*\}? | x\p{AHex}{0,2}`}|${r`o\{[^\}]*\}?`}|${r`\d{1,3}`}`;RegExp(r`
  \\ (?:
    ${i}
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
  | ${a}
  | .
`.replace(/\s+/g,""),"gsu"),RegExp(r`
  \\ (?:
    ${i}
    | .
  )
  | \[:(?:\^?\p{Alpha}+|\^):\]
  | ${a}
  | &&
  | .
`.replace(/\s+/g,""),"gsu");let n=String.raw`\(\?(?:[:=!>A-Za-z\-]|<[=!]|\(DEFINE\))`;Object.freeze({DEFAULT:"DEFAULT",CHAR_CLASS:"CHAR_CLASS"}),RegExp(String.raw`(?<noncapturingStart>${n})|(?<capturingStart>\((?:\?<[^>]+>)?)|\\?.`,"gsu");let o=String.raw`(?:[?*+]|\{\d+(?:,\d*)?\})`;RegExp(String.raw`
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
| (?<qBase>${o})(?<qMod>[?+]?)(?<invalidQ>[?*+\{]?)
| \\?.
`.replace(/\s+/g,""),"gsu");let s=String.raw,c=s`\\g<(?<gRNameOrNum>[^>&]+)&R=(?<gRDepth>[^>]+)>`,l=s`\(\?R=(?<rDepth>[^\)]+)\)|${c}`,g=s`\(\?<(?![=!])(?<captureName>[^>]+)>`;s`${g}|(?<unnamed>\()(?!\?)`,RegExp(s`${g}|${l}|\(\?|\\?.`,"gsu");var p=String.fromCodePoint,h=String.raw,_={},S=globalThis.RegExp;function d(e){let t=p(e);return[t.toLowerCase(),t]}function C(e,t){return(function(e,t){let r=[];for(let a=e;a<=t;a++)r.push(a);return r})(e,t).map(e=>d(e))}_.flagGroups=(()=>{try{new S("(?i:)")}catch{return!1}return!0})(),_.unicodeSets=(()=>{try{new S("[[]]","v")}catch{return!1}return!0})(),_.bugFlagVLiteralHyphenIsRange=!!_.unicodeSets&&(()=>{try{new S(h`[\d\-a]`,"v")}catch{return!0}return!1})(),_.bugNestedClassIgnoresNegation=_.unicodeSets&&new S("[[^a]]","v").test("a"),p(304),p(305),h`[\p{L}\p{M}\p{N}\p{Pc}]`,`C Other
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
XID_Start XIDS`.split(/\s/).map(e=>[e.replace(/[- _]+/g,"").toLowerCase(),e]),p(383),p(383),p(223),p(7838),p(107),p(8490),p(229),p(8491),p(969),p(8486),[d(453),d(456),d(459),d(498),...C(8072,8079),...C(8088,8095),...C(8104,8111),d(8124),d(8140),d(8188)],h`[\p{Alpha}\p{Nd}]`,h`\p{Alpha}`,h`\p{ASCII}`,h`[\p{Zs}\t]`,h`\p{Cc}`,h`\p{Nd}`,h`[\P{space}&&\P{Cc}&&\P{Cn}&&\P{Cs}]`,h`\p{Lower}`,h`[[\P{space}&&\P{Cc}&&\P{Cn}&&\P{Cs}]\p{Zs}]`,h`[\p{P}\p{S}]`,h`\p{space}`,h`\p{Upper}`,h`[\p{Alpha}\p{M}\p{Nd}\p{Pc}]`,h`\p{AHex}`,h`\t`,h`\n`,h`\v`,h`\f`,h`\r`,h`\u2028`,h`\u2029`,h`\uFEFF`,e.s([],18433),e.i(18433);var m=e.i(77313),u=e.i(45546),T=e.i(76114);e.s(["ShikiError",()=>u.ShikiError,"addClassToHast",()=>m.addClassToHast,"applyColorReplacements",()=>T.applyColorReplacements,"codeToHast",()=>m.codeToHast,"codeToHtml",()=>m.codeToHtml,"codeToTokens",()=>m.codeToTokens,"codeToTokensBase",()=>m.codeToTokensBase,"codeToTokensWithThemes",()=>T.codeToTokensWithThemes,"createBundledHighlighter",()=>m.createBundledHighlighter,"createCssVariablesTheme",()=>m.createCssVariablesTheme,"createHighlighterCore",()=>m.createHighlighterCore,"createHighlighterCoreSync",()=>m.createHighlighterCoreSync,"createPositionConverter",()=>m.createPositionConverter,"createShikiInternal",()=>T.createShikiInternal,"createShikiInternalSync",()=>T.createShikiInternalSync,"createShikiPrimitive",()=>T.createShikiPrimitive,"createShikiPrimitiveAsync",()=>T.createShikiPrimitiveAsync,"createSingletonShorthands",()=>m.createSingletonShorthands,"flatTokenVariants",()=>m.flatTokenVariants,"getLastGrammarState",()=>T.getLastGrammarState,"getSingletonHighlighterCore",()=>m.getSingletonHighlighterCore,"getTokenStyleObject",()=>m.getTokenStyleObject,"guessEmbeddedLanguages",()=>m.guessEmbeddedLanguages,"hastToHtml",()=>m.hastToHtml,"isNoneTheme",()=>T.isNoneTheme,"isPlainLang",()=>T.isPlainLang,"isSpecialLang",()=>T.isSpecialLang,"isSpecialTheme",()=>T.isSpecialTheme,"makeSingletonHighlighter",()=>m.makeSingletonHighlighter,"makeSingletonHighlighterCore",()=>m.makeSingletonHighlighterCore,"normalizeGetter",()=>T.normalizeGetter,"normalizeTheme",()=>T.normalizeTheme,"resolveColorReplacements",()=>T.resolveColorReplacements,"splitLines",()=>T.splitLines,"splitToken",()=>m.splitToken,"splitTokens",()=>m.splitTokens,"stringifyTokenStyle",()=>m.stringifyTokenStyle,"toArray",()=>T.toArray,"tokenizeAnsiWithTheme",()=>m.tokenizeAnsiWithTheme,"tokenizeWithTheme",()=>T.tokenizeWithTheme,"tokensToHast",()=>m.tokensToHast,"transformerDecorations",()=>m.transformerDecorations],90423),e.i(90423),e.s(["createHighlighter",()=>t.createHighlighter],24420)}]);