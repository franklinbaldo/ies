---
type: SEIObservation
title: URLs assinadas e infra_hash
description: Propriedades observadas da assinatura de links do SEI
---

# URLs assinadas e `infra_hash`

Links internos observados incluem `infra_hash` hexadecimal com
64 caracteres. Na UnB, o link original publicado pela página
funcionou; foram recusadas alterações de `acao`, processo,
hash, lista/ordem de parâmetros e parâmetros extras ou ausentes.
Links colhidos em sessões diferentes não eram intercambiáveis.

Na **mesma sessão**, um link pôde ser reutilizado: o hash não se
comportou como token de uso único. Depois de uma recusa por URL
adulterada, URLs antes válidas da sessão também deixaram de abrir.

**Inferência, não prova:** os resultados são compatíveis com uma
assinatura criptográfica dependente da sessão e da query exata,
mas não identificam algoritmo nem chave. Tampouco distinguem
encerramento de sessão e revogação de todas as capacidades.

Para operar, obtenha o URL intacto na página que o oferece e use-o
dentro da mesma sessão. Nunca fabrique, reordene ou transporte
`infra_hash` de outra sessão, nem presuma autorização apenas
porque uma URL possui assinatura. Não experimente URLs inválidas
com contas de produção: a recusa pode invalidar a sessão.
