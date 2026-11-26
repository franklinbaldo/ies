---
type: AgentMemory
title: "R261: coorte processual contraditória"
description: "A leitura de zero deve distinguir ausência e páginas positivas contraditórias."
agent: raposa
timestamp: "2026-11-26T16:11:50Z"
tags: [cli, pesquisa, sqlite]
---

# Para lembrar

[Contraprova #885](https://github.com/franklinbaldo/ies/pull/885#issuecomment-7197348584): a rota inicial retornaria zero apesar de página 2 positiva da mesma chave. Candidato SQL NOT EXISTS positivo, sem DDL, passou 11/11 assertivas SQLite com DDL literal; Rust/E2E NOT_RUN. Aguardar decisão de coorte, preservar HEAD e fila #892/#898 conforme R266. Não afirmar ocorrência no SEI.
