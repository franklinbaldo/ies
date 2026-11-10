---
type: AgentMemory
title: "BDD-257-02: POST único de comentário com releitura causal e CI verde"
description: "PR #661 confirmou comentário na bancada sintética por CLI real, HTTPS local, anti-IES e SQLite; RED comprovado e GREEN oficial 7/7, replay de cassette e live não executados."
agent: castor
timestamp: "2026-11-10T11:29:31Z"
tags: [ies, anti-ies, comentarios, bdd, rust, sqlite, cassettes, ci]
---

# Próxima rodada

**Fato:** a [PR #661](https://github.com/franklinbaldo/ies/pull/661) implementa a confirmação de `ies processo comentar NUP --texto T --confirmar` restrita às origens de treinamento, com GET do formulário publicado, um POST, 302 não seguido e releitura da lista pelo id aceito e texto exato. O teste executa a CLI Rust real ↔ HTTPS local ↔ anti-IES ↔ SQLite v33 e outra CLI independente comprova novo comentário e antigo preservado. O [RED #42805414597](https://github.com/franklinbaldo/ies/actions/runs/42805414597) falhou por `--confirmar` ainda recusado antes da rede, com os demais quatro testes de comentários PASS. O [GREEN #42807288999](https://github.com/franklinbaldo/ies/actions/runs/42807288999) passou **7/7** no head `2c4a027edcf0ebebfd91d92d0a5b456b171b201c`; o CI rápido [#42807271986](https://github.com/franklinbaldo/ies/actions/runs/42807271986) também PASS.

**Política:** `origem_de_treinamento::Escrita::ProcessoComentar` não está aprovada para produção, com teste unitário de recusa. Sem segredos ou escrita no SEI, sem novos subcódigos; condição `EfeitoIncerto` é variante Rust tipada e saída pública 3 sem retry.

**Provas distintas:** compatibilidade sintética **PASS**; integridade e estrutura das 76 interações UnB/GO [PASS anterior](https://github.com/franklinbaldo/ies/actions/runs/42786966461); confronto semântico independente de cassettes **NOT_RUN**; treinamento/live **NOT_RUN**. O corpo e o 302 do POST permanecem hipótese do anti-IES, não evidência observada do SEI.

**Cafezinho:** visitei e [compartilhei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6766151739) na #388 que retorno 302 sozinho não prova alteração. O teste precisa tanto da afirmação causal quanto da releitura independente e preservação do estado anterior.

**CONTINUAR:** sincronizar a PR #661 com a `main` por merge verdadeiro após esta memória e renovar CI no head final, sem rebase/force-push. Depois revisão 🦅, integração 🐘/🦡; abrir fatia separada BDD-257-03 para falha de verificação pós-POST (efeito incerto sem repetir). A lista inicialmente vazia/302 e replay bilateral seguem abertos.
