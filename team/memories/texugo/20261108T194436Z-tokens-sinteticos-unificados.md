---
type: AgentMemory
title: "Token sintético único entre Controle, árvore e PDF"
description: "PR #500 integrada com CI oficial 7/7 no head sincronizado; formulário de pesquisa e árvore PDF agora reutilizam INFRA_HASH já compartilhado com barra e calendário."
agent: texugo
timestamp: "2026-11-08T19:44:36.333Z"
tags: [anti-ies, emulacao, rust, antichurn, ci]
---

# Retomada

**Fato:** [PR #500](https://github.com/franklinbaldo/ies/pull/500) integrada por merge verdadeiro em [6599059](https://github.com/franklinbaldo/ies/commit/659905904d11024a51390263ba19d18ac2b3879f), após CI completo 7/7 verde do head `c6d4d53`. `controle_resumido.rs` e `arvore_compartilhada.rs` substituíram tokens `b` e `d` por `protocolo_sintetico::INFRA_HASH`. Testes compararam Controle R, barra e PDF; não houve nova migração, subcódigo ou acesso externo.

**Próxima fatia:** outros geradores sintéticos permanecem em `documento_arvore.rs`, `documentos.rs`, `editor_documento.rs`, `pdf_externo.rs`, `pesquisa_documento.rs`, `processo_envio.rs` e `documento_versoes.rs`. Consolidar em pequenos lotes após verificar que os identificadores diferentes não representam semânticas distintas. Nunca confundir consistência sintética com confronto independente de cassette ou live.

**Cafezinho:** leitura da #388 nesta rodada reiterou que dado explicitamente observado vazio difere de dado não observado; preservar essa distinção nas próximas projeções do anti-IES.
