if exists("b:current_syntax")
  finish
endif

syntax match reqResponseSection /^###\(Name: .*\|Directives\|Response\)$/
syntax match reqResponseSectionMarker /^###/ containedin=reqResponseSection
syntax match reqResponseDirective /^@[A-Za-z0-9_-]\+.*$/
syntax match reqResponseDirectiveName /^@[A-Za-z0-9_-]\+/ containedin=reqResponseDirective
syntax match reqResponseStatus /^HTTP \d\+.*$/
syntax match reqResponseHeader /^[A-Za-z0-9-]\+: .*$/
syntax match reqResponseHeaderName /^[A-Za-z0-9-]\+:/ containedin=reqResponseHeader
syntax region reqResponseJson start=/^{/ end=/}/ contains=reqResponseJsonKey,reqResponseString,reqResponseNumber
syntax match reqResponseJsonKey /"[^"]*"\s*:/ contained
syntax region reqResponseString start=/"/ skip=/\\"/ end=/"/ contained
syntax match reqResponseNumber /\v-?\d+(\.\d+)?/ contained

highlight default link reqResponseSection Special
highlight default link reqResponseSectionMarker Delimiter
highlight default link reqResponseDirective PreProc
highlight default link reqResponseDirectiveName Type
highlight default link reqResponseStatus Statement
highlight default link reqResponseHeader Comment
highlight default link reqResponseHeaderName Identifier
highlight default link reqResponseJsonKey Identifier
highlight default link reqResponseString String
highlight default link reqResponseNumber Number

let b:current_syntax = "req_response"
