(globalThis.TURBOPACK||(globalThis.TURBOPACK=[])).push(["object"==typeof document?document.currentScript:void 0,24420,e=>{"use strict";e.i(96288),e.i(61838);var t=e.i(48232);e.i(88361);let a=String.raw,r=a`\[\^?`,n=`c.? | C(?:-.?)?|${a`[pP]\{(?:\^?[-\x20_]*[A-Za-z][-\x20\w]*\})?`}|${a`x[89A-Fa-f]\p{AHex}(?:\\x[89A-Fa-f]\p{AHex})*`}|${a`u(?:\p{AHex}{4})? | x\{[^\}]*\}? | x\p{AHex}{0,2}`}|${a`o\{[^\}]*\}?`}|${a`\d{1,3}`}`;RegExp(a`
  \\ (?:
    ${n}
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
  | ${r}
  | .
`.replace(/\s+/g,""),"gsu"),RegExp(a`
  \\ (?:
    ${n}
    | .
  )
  | \[:(?:\^?\p{Alpha}+|\^):\]
  | ${r}
  | &&
  | .
`.replace(/\s+/g,""),"gsu");let i=String.raw`\(\?(?:[:=!>A-Za-z\-]|<[=!]|\(DEFINE\))`;Object.freeze({DEFAULT:"DEFAULT",CHAR_CLASS:"CHAR_CLASS"}),RegExp(String.raw`(?<noncapturingStart>${i})|(?<capturingStart>\((?:\?<[^>]+>)?)|\\?.`,"gsu");let o=String.raw`(?:[?*+]|\{\d+(?:,\d*)?\})`;RegExp(String.raw`
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
`.replace(/\s+/g,""),"gsu");let p=String.raw,_=p`\\g<(?<gRNameOrNum>[^>&]+)&R=(?<gRDepth>[^>]+)>`,c=p`\(\?R=(?<rDepth>[^\)]+)\)|${_}`,s=p`\(\?<(?![=!])(?<captureName>[^>]+)>`;p`${s}|(?<unnamed>\()(?!\?)`,RegExp(p`${s}|${c}|\(\?|\\?.`,"gsu");var u=String.fromCodePoint,C=String.raw,g={},d=globalThis.RegExp;function l(e){let t=u(e);return[t.toLowerCase(),t]}function S(e,t){return(function(e,t){let a=[];for(let r=e;r<=t;r++)a.push(r);return a})(e,t).map(e=>l(e))}g.flagGroups=(()=>{try{new d("(?i:)")}catch{return!1}return!0})(),g.unicodeSets=(()=>{try{new d("[[]]","v")}catch{return!1}return!0})(),g.bugFlagVLiteralHyphenIsRange=!!g.unicodeSets&&(()=>{try{new d(C`[\d\-a]`,"v")}catch{return!0}return!1})(),g.bugNestedClassIgnoresNegation=g.unicodeSets&&new d("[[^a]]","v").test("a"),u(304),u(305),C`[\p{L}\p{M}\p{N}\p{Pc}]`,`C Other
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
XID_Start XIDS`.split(/\s/).map(e=>[e.replace(/[- _]+/g,"").toLowerCase(),e]),u(383),u(383),u(223),u(7838),u(107),u(8490),u(229),u(8491),u(969),u(8486),new Map([l(453),l(456),l(459),l(498),...S(8072,8079),...S(8088,8095),...S(8104,8111),l(8124),l(8140),l(8188)]),C`[\p{Alpha}\p{Nd}]`,C`\p{Alpha}`,C`\p{ASCII}`,C`[\p{Zs}\t]`,C`\p{Cc}`,C`\p{Nd}`,C`[\P{space}&&\P{Cc}&&\P{Cn}&&\P{Cs}]`,C`\p{Lower}`,C`[[\P{space}&&\P{Cc}&&\P{Cn}&&\P{Cs}]\p{Zs}]`,C`[\p{P}\p{S}]`,C`\p{space}`,C`\p{Upper}`,C`[\p{Alpha}\p{M}\p{Nd}\p{Pc}]`,C`\p{AHex}`,C`\t`,C`\n`,C`\v`,C`\f`,C`\r`,C`\u2028`,C`\u2029`,C`\uFEFF`,e.i(45546),e.i(76114),e.s([],90423),e.i(90423),e.S([t,"createHighlighter,u"],24420)}]);