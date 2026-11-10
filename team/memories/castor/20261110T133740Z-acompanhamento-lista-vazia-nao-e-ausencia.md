---
type: AgentMemory
title: "Prévia de retirada: vazio observado não é ausência de observação"
description: "PR #666 com BDD-348-01/03 na vertical CLI/anti-IES/SQLite v35; ramo de hash corrigido, lista vazia distinguida e CI ainda não executado no SHA atual."
agent: castor
timestamp: "2026-11-10T13:37:40Z"
tags: [ies, anti-ies, acompanhamento, sqlite, bdd, rust, cassettes, ci]
---

# Retomada

**Fato:** na [PR #666](https://github.com/franklinbaldo/ies/pull/666), para BDD-348-01/03, a CLI Rust real e o anti-IES agora têm bancada HTTPS local com SQLite v35, com hipóteses G+H, apenas H, lista observada vazia e lista não observada. Corrigi o uso de `hash` antes da declaração na árvore e reconciliei com `main@0c2f7aa` por merge verdadeiro; a última extensão de teste estava no head `b42d6bb`.

**Provas:** [inspeção de duas cassettes UnB](https://github.com/franklinbaldo/ies/actions/runs/42823266516) PASS em hash/estrutura, 97 interações. Não houve confronto semântico bilateral; teste CLI↔anti-IES↔SQLite do SHA corrente ainda **NOT_RUN**, porque [CI rápido #42825926299](https://github.com/franklinbaldo/ies/actions/runs/42825926299) estava em fila ao encerrar; CI oficial do SHA corrente também não comprovado. Live **NOT_RUN**. Não inferir GREEN de arquivos escritos ou de inspeção de hashes. Sem produção, credenciais ou subcódigos novos.

**Cafezinho:** visitei e comentei na [#388](https://github.com/franklinbaldo/ies/issues/388): fotografia explícita vazia é dado; ausência da fotografia é desconhecido. A segunda não autoriza dizer `ja_estava=true`.

**CONTINUAR:** conferir CI exato, corrigir falhas, validar E2E e pedir revisão 🦅/integração 🐘/🦡. A retirada `--confirmar` continua bloqueada; BDD-348-02 depende da action e formulário observados.
