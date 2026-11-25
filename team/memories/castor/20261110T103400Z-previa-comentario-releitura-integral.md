---
type: AgentMemory
title: "Prévia de comentário comprovada por segunda CLI, sem POST e sem cortar texto"
description: "Na PR #659, BDD-257-02 ganhou prévia GET e releitura independente no SQLite; CI oficial 7/7 verde, mas confirmação e replay bilateral seguem pendentes."
agent: castor
timestamp: "2026-11-10T10:34:00Z"
tags: [ies, anti-ies, comentarios, bdd, rust, sqlite, cassettes, ci]
---

# Para a próxima rodada

**Fato:** a [PR #659](https://github.com/franklinbaldo/ies/pull/659), head `9e46c447`, passou no [CI oficial #42799742274](https://github.com/franklinbaldo/ies/actions/runs/42799742274) com 7/7 jobs e no CI rápido. A prévia `ies processo comentar NUP --texto T` segue links servidos até o formulário sem POST; outra execução real `ies processo comentarios NUP` relê o mesmo SQLite e confirma comentário preexistente intacto. Corrigi um `trim()` que descartava espaços significativos do texto pedido. A confirmação permanece recusada antes da sessão e **BDD-257-02 ainda é parcial**; BDD-257-03 não foi implementado. A PR está Ready, aberta, sem merge; `behind=0` foi observado antes desta memória na `main`.

**Provas distintas:** IES ↔ anti-IES ↔ SQLite: PASS sintético no head; cassettes UnB/GO (76 interações) com [hash e estrutura verificados](https://github.com/franklinbaldo/ies/actions/runs/42786966461), mas replay semântico independente NOT_RUN; treinamento/live NOT_RUN. Nenhuma escrita no SEI. O RED inicial do comando inexistente foi observado; a extensão de releitura e preservação de espaços não teve RED separado.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388); a conversa sobre diferenças entre ausência e ambiguidade reforça distinguir a frase "não gravei" da **releitura de estado**. Isso é orientação para testar, não prova de fidelidade externa.

**CONTINUAR:** revisão 🦅/integração 🐘/🦡 da #659, com eventual merge real da `main` e revalidação no head reconciliado; depois uma nova fatia para POST único, resposta de aceite e releitura causal do comentário, mais efeito incerto sem retry. Não atualizar o BDD canônico para completo com base apenas na prévia.
