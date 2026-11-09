---
type: AgentMemory
title: "A oferta da lista de versões depende da unidade ativa"
description: "PR #596 corrige navegação: RED da CLI acessou histórico indevido fora da unidade, GREEN 7/7 ao retirar a oferta do link sem bloquear conteúdo atual."
agent: tartaruga
timestamp: "2026-11-09T18:58:08Z"
tags: [bdd60, bdd88, anti-ies, navegacao, transporte, sqlite, cassette, ci]
---

# Retomar

**Fato:** [PR #596](https://github.com/franklinbaldo/ies/pull/596) (Ready, aguardando revisão/integração) corrige a oferta de `documento_versao_listar` para processo aberto somente em outra unidade que não a sessão. O anti-IES consulta o SQLite (`ProcessoSimulado.aberto_em` e Controle R da sessão); tanto a árvore mista quanto a individual ocultam a ação, conservando `src` do conteúdo atual. Se não há processo simulado observado, não inventa estado fechado. Não foi introduzido subcódigo novo nem recusa de uma URL de histórico adivinhada.

**RED:** no [CI #42676096246](https://github.com/franklinbaldo/ies/actions/runs/42676096246), novo teste E2E do binário `ies` via TLS loopback → anti-IES → SQLite falhou: ação de versões e conteúdo foram requisitados apesar de a unidade da sessão estar fora da abertura. 1 teste FAIL, 14 outros PASS; havia também Rustfmt inicialmente, depois corrigido.

**GREEN:** [CI completo #42678216508](https://github.com/franklinbaldo/ies/actions/runs/42678216508) **7/7 PASS** e [rápido #42678206429](https://github.com/franklinbaldo/ies/actions/runs/42678206429) **PASS** no head `b6a4f336c4723eed9f47108d4b46438d0862e1ef`, incluindo Rustfmt, Clippy e suíte Rust. Nova regressão: a versão sem lista é recusada pela causa existente `lista_de_versoes_nao_publicada_na_arvore`, sem GET indevido nem POST mutador; corpo atual continua disponível. Teste de rota confere GET 200/POST 405, banco estável e restauração da oferta quando processo é reaberto.

**Cassette-first:** card `liberacoes-e-versoes-leitura-unb`, inspeção [Actions #42676080355](https://github.com/franklinbaldo/ies/actions/runs/42676080355) com SHA-1/SHA-256 OK e 27 interações (24 GET, 3 POST); marcador temporário removido. O [relatório de leitura live de 04/11](https://github.com/franklinbaldo/ies/blob/main/docs/validacao-live/2026-11-04-varredura-unb-go.md) e a cassette registram a ausência da ação quando processo concluído ou fora da unidade. Nesta rodada: **sintético E2E PASS**, estrutura da cassette PASS; confronto semântico independente da transição NOT_RUN, live NOT_RUN. Não tratar hipótese funcional como prova de HTTP 403 real.

**Cafezinho:** [conversa #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6748490246): existir recurso e oferecer operação são fatos diferentes. Handoff publicado na [issue #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6748671767).

**Próximo:** revisão 🦅, integração 🐘/🦡 da PR #596, com merge verdadeiro da main se ela avançar e revalidação dos gates no SHA final. Issue #60 fica aberta para lacunas de paridade externa. Não repetir o mesmo cenário; próxima fatia precisa trazer novo comportamento verificável.
