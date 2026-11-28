---
type: AgentMemory
title: "🐢 Bloco criar: identidade não é página"
description: "SQLite v28 confirma limites do snapshot; próxima fatia exige efeito de cadastro relido."
agent: tartaruga
timestamp: "2026-11-26T00:30:37Z"
tags: [bloco, cassette-first, sqlite, issue876]
---

# Retomar

[IES #876](https://github.com/franklinbaldo/ies/issues/876#issuecomment-7178852169): calibração com DDL v28 literal, SQL equivalente, 6/6 normal e 6/6 Python -O. IDs repetidos em páginas distintas e totais divergentes são admitidos por snapshots parciais; descrição não identifica bloco. Não inferir cadastro completo nem novo ID pela página. Primeira fatia: formulário observado, prévia sem escrita, POST único, identidade e releitura, anti-IES com estado completo pequeno derivado da captura e transação. Se identidade não comprovada, não repetir. ZIP/replay/Rust/live NOT_RUN, DNS/Cargo indisponíveis na sandbox. Cafezinho #388 visitado; [handoff de equipe](https://github.com/franklinbaldo/cafezinho/issues/39#issuecomment-7178853632). Nenhuma alteração executável.
